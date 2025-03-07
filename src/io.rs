//! Functionality related to input and output.

use core::{
    fmt::{Arguments, Write},
    time::Duration,
};

use crate::{
    Errno, SyscallNum,
    consts::{PIT_IRQ_PERIOD, STDERR, STDOUT},
    data::NullTermStr,
    fs::{FileDescriptor, OpenFlags, open_no_create, read_byte, write_byte},
    nulltermstr, syscall,
    thread::sleep,
};

#[cfg(not(debug_assertions))]
/// Path to the Linux system console device.
const DEV_CONSOLE_PATH: NullTermStr<13> = nulltermstr!(b"/dev/console");
#[cfg(debug_assertions)]
/// Path to the Linux system console device.
const DEV_CONSOLE_PATH: NullTermStr<9> = nulltermstr!(b"/dev/tty");
/// Byte representing the "backspace" character.
const BACKSP_BYTE: u8 = 8;

/// Print to stdout using format syntax.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (
        $crate::io::__print_str(format_args!($($arg)*))
    );
}

/// Print, with a newline, to stdout using format syntax.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

/// Print to stderr using format syntax.
#[macro_export]
macro_rules! eprint {
    ($($arg:tt)*) => (
        $crate::io::__print_err(format_args!($($arg)*))
    );
}

/// Print, with a newline, to stderr using format syntax.
#[macro_export]
macro_rules! eprintln {
    () => ($crate::eprint!("\n"));
    ($($arg:tt)*) => ($crate::eprint!("{}\n", format_args!($($arg)*)));
}

/// Represents stdout.
#[derive(Debug)]
struct Stdout;

/// Represents stderr.
#[derive(Debug)]
struct Stderr;

macro_rules! write_str_impl {
    [$(($t:ty, $fd:expr)),*] => {
        $(impl Write for $t {
            fn write_str(&mut self, s: &str) -> core::fmt::Result {
                // SAFETY: The caller is only able to choose the string itself. Everything else
                // is chosen statically or based on the string length.
                unsafe {
                    syscall!(SyscallNum::Write, $fd, s.as_ptr() as usize, s.len());
                }
                Ok(())
            }
        })*
    };
}
write_str_impl![(Stdout, STDOUT), (Stderr, STDERR)];

/// For [`print`] and [`println`] use only.
#[doc(hidden)]
pub fn __print_str(args: Arguments<'_>) {
    Stdout.write_fmt(args).unwrap();
}

/// For [`eprint`] and [`eprintln`] use only.
#[doc(hidden)]
pub fn __print_err(args: Arguments<'_>) {
    Stderr.write_fmt(args).unwrap();
}

/// Struct to read from and write to the [system console](https://en.wikipedia.org/wiki/Linux_console).
/// Contains a file descriptor for the system console.
#[derive(Debug)]
pub struct Console(FileDescriptor);
impl Console {
    /// Opens the [system console](https://en.wikipedia.org/wiki/Linux_console), returning its file
    /// descriptor as a [`Console`].
    ///
    /// # Errors
    ///
    /// This function propagates any errors from the underlying [`open_no_create`].
    pub fn open_console() -> Result<Self, Errno> {
        open_no_create(
            &DEV_CONSOLE_PATH,
            &(OpenFlags::O_RDWR | OpenFlags::O_NDELAY),
        )
        .map(Self)
    }

    /// Reads a single byte from the [system console](https://en.wikipedia.org/wiki/Linux_console),
    /// looping until a byte is read.
    ///
    /// # Errors
    ///
    /// This function propagates any errors from the underlying [`read_byte`] and [`sleep`].
    pub fn read_console_byte(&self) -> Result<u8, Errno> {
        let dur = Duration::from_nanos(PIT_IRQ_PERIOD);
        loop {
            let result = read_byte(self.0)?;
            match result {
                Some(0) | None => sleep(&dur)?, // Nothing read, sleep then loop again...
                Some(byte) => return Ok(byte),  // Got a byte! Return it
            }
        }
    }

    /// Writes a single byte to the [system console](https://en.wikipedia.org/wiki/Linux_console),
    /// returning the number of bytes written.
    ///
    /// # Errors
    ///
    /// This function propagates any errors from the underlying [`write_byte`] function.
    pub fn write_console_byte(&self, byte: u8) -> Result<usize, Errno> {
        write_byte(self.0, byte)
    }

    /// Read a line of up to `N` bytes from the console.
    ///
    /// # Errors
    ///
    /// This function propagates any errors from the underlying [`Self::read_console_byte`] and
    /// [`Self::write_console_byte`] functions.
    pub fn read_line<const N: usize>(&self) -> Result<[u8; N], Errno> {
        let mut result = [0x00; N];

        let mut i: usize = 0;
        while i < N {
            let read_byte = self.read_console_byte()?;
            // We want to see what we're typing!
            self.write_console_byte(read_byte)?;
            result[i] = read_byte;

            if read_byte == BACKSP_BYTE {
                // Handle backspace
                result[i] = 0;
                // Don't advance the index for backspace!
            } else if read_byte == b'\n' {
                // Handle newline
                result[i] = 0;
                // End early
                return Ok(result);
            } else {
                // Advance index and continue
                i += 1;
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests;
