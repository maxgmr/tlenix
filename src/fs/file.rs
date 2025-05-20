//! This module is responsible for the [`File`] type and all associated file operations.

use crate::{
    Errno, SyscallNum,
    fs::{FileDescriptor, FileStat, FileStatRaw, LseekWhence, OpenOptions},
    syscall_result,
};

/// An object providing access to an open file on the filesystem.
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct File {
    #[allow(clippy::struct_field_names)]
    file_descriptor: FileDescriptor,
    open_options: OpenOptions,
}
impl File {
    /// Creates a [`File`] at the given [`FileDescriptor`] with the given open options. Not
    /// intended to be used directly.
    #[doc(hidden)]
    #[must_use]
    pub(crate) fn __new(file_descriptor: FileDescriptor, open_options: &OpenOptions) -> Self {
        Self {
            file_descriptor,
            open_options: open_options.clone(),
        }
    }

    /// Gets information about this [`File`] in the form of a [`FileStat`].
    ///
    /// Wrapper around the [`fstat`](https://man7.org/linux/man-pages/man2/fstat.2.html) Linux
    /// syscall.
    ///
    /// # Errors
    ///
    /// This function propagates any [`Errno`]s from the underlying `fstat` Linux syscall. It
    /// also returns [`Errno::Eio`] if it gets malformed data from the syscall itself.
    pub fn stat(&self) -> Result<FileStat, Errno> {
        let mut stats = FileStatRaw::default();

        // SAFETY: Arguments are correct. `stats_ptr` is valid at the time of calling and is
        // dropped right afterwards.
        unsafe {
            syscall_result!(SyscallNum::Fstat, self.file_descriptor, &raw mut stats)?;
        }
        stats.try_into()
    }

    /// Reads bytes from the [`File`] into the given buffer. Returns the number of bytes read from
    /// the file on success.
    ///
    /// This function also advances the internal file cursor.
    ///
    /// Wrapper around the [`read`](https://www.man7.org/linux/man-pages/man2/read.2.html) Linux
    /// syscall.
    ///
    /// # Errors
    ///
    /// This function returns an [`Errno`] if the underlying `read` syscall fails.
    pub fn read(&self, buffer: &mut [u8]) -> Result<usize, Errno> {
        let buf_ptr = buffer.as_mut_ptr();

        // SAFETY: The arguments are correct and the length is guaranteed to match the given
        // buffer. The mutable raw pointer to the array is not accessed after mutating the array
        // and goes out of scope right after reading.
        unsafe {
            syscall_result!(
                SyscallNum::Read,
                self.file_descriptor,
                buf_ptr,
                buffer.len()
            )
        }
    }

    /// Reads a single byte from the file.
    ///
    /// Will return [`None`] if the end of the file has been reached.
    ///
    /// Internally relies on the [`read`](https://www.man7.org/linux/man-pages/man2/read.2.html)
    /// Linux syscall.
    ///
    /// # Errors
    ///
    /// Will propagate any [`Errno`]s returned from the call to `read`.
    pub fn read_byte(&self) -> Result<Option<u8>, Errno> {
        let mut byte: u8 = u8::default();

        // SAFETY: The file descriptor is tied to the file itself. The mutable raw pointer to
        // `byte` is dropped at the end of the function, so there is no risk of a
        // use-after-free. All other arguments are statically-chosen and correct.
        let bytes_read =
            unsafe { syscall_result!(SyscallNum::Read, self.file_descriptor, &raw mut byte, 1)? };

        // End of file.
        if bytes_read == 0 {
            return Ok(None);
        }

        Ok(Some(byte))
    }

    /// Writes bytes from the provided buffer to the given file, starting at the file's internal
    /// cursor location. Returns the number of bytes written on success.
    ///
    /// If this file was opened with [`OpenOptions::append`], then the bytes will always be written
    /// to the end of the file.
    ///
    /// Relies on the [`write`](https://www.man7.org/linux/man-pages/man2/write.2.html) Linux
    /// syscall internally.
    ///
    /// # Errors
    ///
    /// This function propagates any errors encountered during the `write` syscall, returning an
    /// [`Errno`].
    pub fn write(&self, buffer: &[u8]) -> Result<usize, Errno> {
        let mut total_bytes_written = 0;

        while total_bytes_written < buffer.len() {
            // SAFETY: The arguments are correct. The raw pointer to the buffer is dropped
            // before the buffer goes out of scope. The buffer length is guaranteed to be correct.
            total_bytes_written += unsafe {
                syscall_result!(
                    SyscallNum::Write,
                    self.file_descriptor,
                    buffer.as_ptr(),
                    buffer.len()
                )?
            };
        }

        Ok(total_bytes_written)
    }

    /// Writes a single byte to the file. Returns the number of bytes written.
    ///
    /// Internally relies on the [`write`](https://www.man7.org/linux/man-pages/man2/write.2.html)
    /// Linux syscall.
    ///
    /// # Errors
    ///
    /// Will propagate any [`Errno`]s returned from the call to `write`.
    pub fn write_byte(&self, byte: u8) -> Result<usize, Errno> {
        // SAFETY: The pointer to the byte is valid. The buffer size is statically-chosen and
        // matches the single byte being written. Any issues with user-given arguments are handled
        // gracefully by the underlying syscall.
        unsafe { syscall_result!(SyscallNum::Write, self.file_descriptor, &raw const byte, 1) }
    }

    /// Gets the current cursor location within the [`File`].
    ///
    /// Uses the [`lseek`](https://www.man7.org/linux/man-pages/man2/lseek.2.html) Linux syscall
    /// internally.
    ///
    /// # Errors
    ///
    /// This function propagates any errors encountered during the underlying `lseek` operation.
    pub fn cursor(&self) -> Result<usize, Errno> {
        self.cursor_offset(0)
    }

    /// Offsets the cursor from its current location by the given number. Returns the new cursor
    /// location.
    ///
    /// Uses the [`lseek`](https://www.man7.org/linux/man-pages/man2/lseek.2.html) Linux syscall
    /// internally.
    ///
    /// # Errors
    ///
    /// This function propagates any errors encountered during the underlying `lseek` operation.
    pub fn cursor_offset(&self, offset: i64) -> Result<usize, Errno> {
        self.lseek_wrapper(offset, LseekWhence::SeekCur)
    }

    /// Sets the cursor to `offset` bytes. Returns the new cursor location.
    ///
    /// Uses the [`lseek`](https://www.man7.org/linux/man-pages/man2/lseek.2.html) Linux syscall
    /// internally.
    ///
    /// # Errors
    ///
    /// This function propagates any errors encountered during the underlying `lseek` operation.
    pub fn set_cursor(&self, offset: i64) -> Result<usize, Errno> {
        self.lseek_wrapper(offset, LseekWhence::SeekSet)
    }

    /// Sets the cursor to the end of the file. Returns the new cursor location.
    ///
    /// Uses the [`lseek`](https://www.man7.org/linux/man-pages/man2/lseek.2.html) Linux syscall
    /// internally.
    ///
    /// # Errors
    ///
    /// This function propagates any errors encountered during the underlying `lseek` operation.
    pub fn cursor_to_end(&self) -> Result<usize, Errno> {
        self.cursor_to_end_offset(0)
    }

    /// Sets the cursor to the end of the file, plus an offset. Returns the new cursor location.
    ///
    /// Uses the [`lseek`](https://www.man7.org/linux/man-pages/man2/lseek.2.html) Linux syscall
    /// internally.
    ///
    /// # Errors
    ///
    /// This function propagates any errors encountered during the underlying `lseek` operation.
    pub fn cursor_to_end_offset(&self, offset: i64) -> Result<usize, Errno> {
        self.lseek_wrapper(offset, LseekWhence::SeekEnd)
    }

    /// Wrapper around the `lseek` syscall to reduce code duplication.
    fn lseek_wrapper(&self, offset: i64, whence: LseekWhence) -> Result<usize, Errno> {
        // SAFETY: The `offset` argument matches the C `off_t` type. The `whence` argument is
        // restricted to the allowed values by the `LseekWhence` enum.
        unsafe { syscall_result!(SyscallNum::Lseek, self.file_descriptor, offset, whence) }
    }
}
