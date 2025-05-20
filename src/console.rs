//! Handles the [`Console`] struct, which gives read and write access to the
//! [system console](https://en.wikipedia.org/wiki/Linux_console).

use crate::{
    Errno,
    fs::{File, OpenOptions},
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
    /// Opens the system console.
    ///
    /// # Errors
    ///
    /// This function propagates any I/O errors associated with opening the system console device
    /// file.
    pub fn open_console() -> Result<Self, Errno> {
        Ok(Self(OpenOptions::new().read_write().open(CONSOLE_PATH)?))
    }
}
