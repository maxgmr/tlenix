//! Handles the [`Console`] struct, which gives read and write access to the
//! [system console](https://en.wikipedia.org/wiki/Linux_console).

use alloc::vec::Vec;
use core::time::Duration;

use crate::{
    Errno,
    fs::{File, FileType, OpenOptions},
    thread,
};

#[cfg(not(debug_assertions))]
/// Path to the Linux system console device.
const CONSOLE_PATH: &str = "/dev/console";
#[cfg(debug_assertions)]
/// Path to the Linux system console device.
const CONSOLE_PATH: &str = "/dev/tty";

/// Byte representing a backspace.
const BACKSPACE_BYTE: u8 = 8;
/// Byte representing a newline.
const NEWLINE_BYTE: u8 = b'\n';
/// Byte representing a backslash.
const BACKSLASH_BYTE: u8 = b'\\';

/// Struct to read from and write to the
/// [system console](https://en.wikipedia.org/wiki/Linux_console). Contains a file descriptor for
/// the system console.
#[derive(Debug)]
pub struct Console(File);
impl Console {
    /// Opens the system console in non-blocking mode with read and write permissions.
    ///
    /// # Errors
    ///
    /// This function propagates any I/O errors associated with opening the system console device
    /// file.
    ///
    /// Additionally, this function will return [`Errno::Enotty`] if the character device is
    /// missing from the filesystem.
    pub fn open() -> Result<Self, Errno> {
        let file = OpenOptions::new()
            .read_write()
            .non_blocking(true)
            .open(CONSOLE_PATH)?;

        // Reject if not a character device
        if FileType::CharacterDevice != file.stat()?.file_type {
            return Err(Errno::Enotty);
        }

        Ok(Self(file))
    }

    /// Reads a single byte from the [system console](https://en.wikipedia.org/wiki/Linux_console),
    /// looping until a byte is read.
    ///
    /// # Errors
    ///
    /// This function propagates any errors from the underlying calls to [`File::read_byte`] and
    /// [`thread::sleep`].
    pub fn read_byte(&self) -> Result<u8, Errno> {
        let sleep_duration = Duration::from_nanos(thread::PIT_IRQ_PERIOD);
        loop {
            match self.0.read_byte() {
                // Nothing read; sleep then try again
                Ok(None) | Err(Errno::Eagain) => thread::sleep(&sleep_duration)?,
                // Propagate non-retryable errors
                Err(e) => return Err(e),
                // Got a byte! Return it!
                Ok(Some(b)) => return Ok(b),
            }
        }
    }

    /// Writes a single byte to the [system console](https://en.wikipedia.org/wiki/Linux_console),
    /// returning the number of bytes written.
    ///
    /// # Errors
    ///
    /// This function propagates any errors from the underlying [`File::write_byte`] function.
    pub fn write_byte(&self, byte: u8) -> Result<usize, Errno> {
        self.0.write_byte(byte)
    }

    /// Reads a line from the console (up to a maximum size).
    ///
    /// # Errors
    ///
    /// This function propagates any errors from the underlying [`Self::read_byte`] and
    /// [`Self::write_byte`] functions.
    pub fn read_line(&self, max: usize) -> Result<Vec<u8>, Errno> {
        let mut result = Vec::new();

        let mut last_was_backslash = false;
        while result.len() < max {
            match self.read_byte()? {
                NEWLINE_BYTE => {
                    // newline; return right away
                    if last_was_backslash {
                        // Escaped newline
                        result.push(NEWLINE_BYTE);
                    } else {
                        return Ok(result);
                    }
                }
                BACKSLASH_BYTE => {
                    last_was_backslash = true;
                    continue;
                }
                BACKSPACE_BYTE => {
                    result.pop();
                }
                new_byte => result.push(new_byte),
            }
            last_was_backslash = false;
        }
        Ok(result)
    }
}
