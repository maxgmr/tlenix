//! Functionality related to printing strings to standard output.

use core::fmt::{Arguments, Write};

use crate::{SyscallNum, syscall_result};

/// The standard output stream.
const STDOUT: usize = 1;
/// The standard error stream.
const STDERR: usize = 2;

/// Represents the standard output stream.
#[derive(Debug)]
struct Stdout;

/// Represents the standard error stream.
#[derive(Debug)]
struct Stderr;

// Macro to implement [`Write`] for Stdout and Stderr.
macro_rules! write_str_impl {
    [$(($t:ty, $fd:expr)),*] => {
       $(impl Write for $t {
           fn write_str(&mut self, s: &str) -> core::fmt::Result {
                // SAFETY: The caller is only able to choose the string itself. Everything else
                // is chosen statically or based on the string length.
                unsafe {
                    syscall_result!(SyscallNum::Write, $fd, s.as_ptr() as usize, s.len())
                        .map_err(|_| core::fmt::Error {})?;
                }
                Ok(())
           }
       })*
    };
}
write_str_impl!((Stdout, STDOUT), (Stderr, STDERR));

/// For [`print`] and [`println`] use only.
#[doc(hidden)]
pub fn __print_str(args: Arguments<'_>) {
    #[allow(clippy::unwrap_used)]
    Stdout.write_fmt(args).unwrap();
}

/// For [`eprint`] and [`eprintln`] use only.
#[doc(hidden)]
pub fn __print_err(args: Arguments<'_>) {
    #[allow(clippy::unwrap_used)]
    Stderr.write_fmt(args).unwrap();
}

/// Print to the standard output using Rust format syntax.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::__print_str(format_args!($($arg)*)));
}

/// Print, with a newline, to the standard output using Rust format syntax.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

/// Print to the standard error stream using Rust format syntax.
#[macro_export]
macro_rules! eprint {
    ($($arg:tt)*) => ($crate::__print_err(format_args!($($arg)*)));
}

/// Print, with a newline, to the standard error stream using Rust format syntax.
#[macro_export]
macro_rules! eprintln {
    () => ($crate::eprint!("\n"));
    ($($arg:tt)*) => ($crate::eprint!("{}\n", format_args!($($arg)*)));
}

#[cfg(test)]
mod tests;
