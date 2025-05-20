//! Handles the [`Console`] struct, which gives read and write access to the
//! [system console](https://en.wikipedia.org/wiki/Linux_console).

use core::time::Duration;

use crate::{
    Errno, PIT_IRQ_PERIOD,
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
        let sleep_duration = Duration::from_nanos(PIT_IRQ_PERIOD);
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
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test_case]
    fn open_console() {
        let _ = Console::open().unwrap();
    }
}
