//! Handles the [`Console`] struct, which gives read and write access to the
//! [system console](https://en.wikipedia.org/wiki/Linux_console).

use crate::{
    Errno,
    fs::{File, FileType, OpenOptions},
};

// Path to the Linux system console device.
#[cfg(not(debug_assertions))]
const CONSOLE_PATH: &str = "/dev/console";
#[cfg(debug_assertions)]
const CONSOLE_PATH: &str = "/dev/tty";

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
