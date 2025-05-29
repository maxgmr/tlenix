//! This module is responsible for the [`File`] type and all associated file operations.

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use core::mem::size_of;

use crate::{
    Errno, NULL_BYTE, NixString, PAGE_SIZE, SyscallNum,
    fs::{
        DirEnt, FileDescriptor, FileStat, LseekWhence, OpenOptions, types::DirEntRawHeader,
        types::FileStatRaw,
    },
    syscall, syscall_result,
};

use super::types::DirEntType;

/// Buffer for reading directory entries. Uses page size for better performance.
const DIR_ENT_BUF_SIZE: usize = PAGE_SIZE;

/// An object providing access to an open file on the filesystem.
#[derive(Debug, PartialEq, Hash)]
pub struct File {
    #[allow(clippy::struct_field_names)]
    file_descriptor: FileDescriptor,
    open_options: OpenOptions,
}
impl File {
    /// Statically defines a [`File`] with the given [`FileDescriptor`]. Used to create the
    /// standard streams.
    #[doc(hidden)]
    #[must_use]
    pub(crate) const fn define(file_descriptor: FileDescriptor) -> Self {
        Self {
            file_descriptor,
            open_options: OpenOptions::dummy(),
        }
    }

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

    /// Reads the entire contents of this file into a [`Vec<u8>`].
    ///
    /// Convenience function. Uses [`Self::read`] internally.
    ///
    /// This function tries to keep the file cursor at the same spot it was before this function
    /// was called.
    ///
    /// # Errors
    ///
    /// This function will propagate any [`Errno`]s from the internal call to [`Self::read`].
    pub fn read_to_bytes(&self) -> Result<Vec<u8>, Errno> {
        let mut buffer = Vec::new();
        // Chunks are page size for better performance
        let mut chunk = [0_u8; PAGE_SIZE];

        let orig_cursor = self.cursor()?;

        loop {
            match self.read(&mut chunk) {
                // EOF
                Ok(0) => break,
                // Got more bytes!
                Ok(num_bytes_read) => {
                    buffer.extend_from_slice(&chunk[..num_bytes_read]);
                }
                // Error
                Err(errno) => {
                    // We have to allow it to be unused, this is simply a last-ditch effort to
                    // restore the cursor after already failing.
                    #[allow(clippy::cast_possible_wrap, unused_must_use)]
                    if let Some(orig_cursor) = orig_cursor {
                        self.set_cursor(orig_cursor as i64);
                    }
                    return Err(errno);
                }
            }
        }

        // Restore original cursor location
        #[allow(clippy::cast_possible_wrap)]
        if let Some(orig_cursor) = orig_cursor {
            self.set_cursor(orig_cursor as i64)?;
        }

        Ok(buffer)
    }

    /// Reads the entire contents of this file into a [`String`].
    ///
    /// Convenience function. Uses [`Self::read`] internally.
    ///
    /// This function tries to keep the file cursor at the same spot it was before this function
    /// was called.
    ///
    /// # Errors
    ///
    /// This function will return [`Errno::Eilseq`] if the bytes of the file are not valid UTF-8.
    ///
    /// This function will propagate any [`Errno`]s from the internal call to [`Self::read`].
    pub fn read_to_string(&self) -> Result<String, Errno> {
        String::from_utf8(self.read_to_bytes()?).map_err(|_| Errno::Eilseq)
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
            let remaining_bytes = &buffer[total_bytes_written..];
            // SAFETY: The arguments are correct. The raw pointer to the buffer is dropped
            // before the buffer goes out of scope. The buffer length is guaranteed to be correct.
            total_bytes_written += unsafe {
                syscall_result!(
                    SyscallNum::Write,
                    self.file_descriptor,
                    remaining_bytes.as_ptr(),
                    remaining_bytes.len()
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

    /// Gets the entries of this directory.
    ///
    /// Naturally, this function is only usable if this [`File`] is a directory. Otherwise,
    /// [`Errno::Enotdir`] will be returned.
    ///
    /// Once this function completes operation, it will return the file cursor back to the point it
    /// was when this function was called.
    ///
    /// Uses the [`getdents64`](https://www.man7.org/linux/man-pages/man2/getdents.2.html) Linux
    /// syscall internally.
    ///
    /// # Errors
    ///
    /// This function returns [`Errno::Enotdir`] if this [`File`] is not a directory.
    ///
    /// This function propagates any [`Errno`]s returned by the underlying `getdents64`,
    /// [`File::cursor`], or [`File::set_cursor`] calls.
    pub fn dir_ents(&self) -> Result<Vec<DirEnt>, Errno> {
        /// Offset of the directory entry name from the start of its bytes.
        const NAME_OFFSET: usize = size_of::<DirEntRawHeader>();

        let orig_cursor = self.cursor()?;

        let mut results: Vec<DirEnt> = Vec::new();
        let mut buf = [0_u8; DIR_ENT_BUF_SIZE];

        // Keep reading entries until there's nothing left to read
        loop {
            // SAFETY: The file descriptor is tied to this struct. The length of the buffer is
            // programmatically-determined and guaranteed to match the actual buffer length.
            let bytes_read = match unsafe {
                syscall_result!(
                    SyscallNum::Getdents64,
                    self.file_descriptor,
                    buf.as_mut_ptr(),
                    buf.len()
                )
            } {
                Ok(bytes_read) => bytes_read,
                Err(errno) => {
                    // Attempt to restore the original cursor before returning the error.
                    // We're suppressing this warning here because we care more about returning a
                    // helpful error message. If the cursor set fails _too_, then it's likely
                    // caused by the original error in the first place, so we don't care as much
                    // about returning the set_cursor error.
                    #[allow(unused_must_use)]
                    if let Some(orig_cursor) = orig_cursor {
                        // We have to allow it to be unused, this is simply a last-ditch effort to
                        // restore the cursor after already failing.
                        #[allow(clippy::cast_possible_wrap, unused_must_use)]
                        self.set_cursor(orig_cursor as i64);
                    }
                    return Err(errno);
                }
            };

            // If `getdents64` has nothing left to give, we're done!
            if bytes_read == 0 {
                break;
            }

            // Keep reading raw dir ent headers (and their name strings) until we reach the end of
            // the returned bytes
            let mut offset = 0;
            while offset < bytes_read {
                // SAFETY: `getdents64` guarantees data won't be written past the end of `buf`. The
                // DirEntRawHeader layout matches the bytes returned by `getdents64`.
                // read_unaligned() handles cases where the bytes could be unaligned.
                let raw_header: DirEntRawHeader = unsafe {
                    buf.as_ptr()
                        .add(offset)
                        .cast::<DirEntRawHeader>()
                        .read_unaligned()
                };

                // Slice for this particular directory entry.
                let entry_slice = &buf[offset..(offset + raw_header.d_reclen as usize)];
                let name_bytes = &entry_slice[NAME_OFFSET..];
                let name_end = name_bytes
                    .iter()
                    .position(|&byte| byte == NULL_BYTE)
                    .unwrap_or(name_bytes.len());
                let name = str::from_utf8(&name_bytes[..name_end])
                    .map_err(|_| Errno::Eilseq)?
                    .to_string();

                offset += raw_header.d_reclen as usize;

                results.push(DirEnt::from_raw(raw_header, name));
            }
        }

        // Reset the cursor to its original state.
        if let Some(orig_cursor) = orig_cursor {
            #[allow(clippy::cast_possible_wrap)]
            self.set_cursor(orig_cursor as i64)?;
        }

        Ok(results)
    }

    /// Checks whether or not this [`File`] is an empty directory.
    ///
    /// # Errors
    ///
    /// This function will return an [`Errno::Enotdir`] if this [`File`] is not a directory at all.
    ///
    /// This function will propagate any [`Errno`]s returned by the underlying call to
    /// [`File::dir_ents`].
    pub fn is_dir_empty(&self) -> Result<bool, Errno> {
        let dir_ents = self.dir_ents()?;

        if dir_ents.len() > 2 {
            return Ok(false);
        }

        // An empty dir can only contain entries for itself and its parent.
        for dent in dir_ents {
            match (dent.name.as_str(), dent.d_type) {
                ("." | "..", DirEntType::Dir) => {}
                _ => return Ok(false),
            }
        }

        Ok(true)
    }

    /// Gets the current cursor location within the [`File`].
    ///
    /// Returns [`None`] if cursor operations do not apply to this [`File`]; i.e., the file is a
    /// terminal, socket, pipe, or FIFO.
    ///
    /// Uses the [`lseek`](https://www.man7.org/linux/man-pages/man2/lseek.2.html) Linux syscall
    /// internally.
    ///
    /// # Errors
    ///
    /// This function propagates any errors encountered during the underlying `lseek` operation.
    pub fn cursor(&self) -> Result<Option<usize>, Errno> {
        self.cursor_offset(0)
    }

    /// Offsets the cursor from its current location by the given number. Returns the new cursor
    /// location.
    ///
    /// Returns [`None`] if cursor operations do not apply to this [`File`]; i.e., the file is a
    /// terminal, socket, pipe, or FIFO.
    ///
    /// Uses the [`lseek`](https://www.man7.org/linux/man-pages/man2/lseek.2.html) Linux syscall
    /// internally.
    ///
    /// # Errors
    ///
    /// This function propagates any errors encountered during the underlying `lseek` operation.
    pub fn cursor_offset(&self, offset: i64) -> Result<Option<usize>, Errno> {
        self.lseek_wrapper(offset, LseekWhence::SeekCur)
    }

    /// Sets the cursor to `offset` bytes. Returns the new cursor location.
    ///
    /// Returns [`None`] if cursor operations do not apply to this [`File`]; i.e., the file is a
    /// terminal, socket, pipe, or FIFO.
    ///
    /// Uses the [`lseek`](https://www.man7.org/linux/man-pages/man2/lseek.2.html) Linux syscall
    /// internally.
    ///
    /// # Errors
    ///
    /// This function propagates any errors encountered during the underlying `lseek` operation.
    pub fn set_cursor(&self, offset: i64) -> Result<Option<usize>, Errno> {
        self.lseek_wrapper(offset, LseekWhence::SeekSet)
    }

    /// Sets the cursor to the end of the file. Returns the new cursor location.
    ///
    /// Returns [`None`] if cursor operations do not apply to this [`File`]; i.e., the file is a
    /// terminal, socket, pipe, or FIFO.
    ///
    /// Uses the [`lseek`](https://www.man7.org/linux/man-pages/man2/lseek.2.html) Linux syscall
    /// internally.
    ///
    /// # Errors
    ///
    /// This function propagates any errors encountered during the underlying `lseek` operation.
    pub fn cursor_to_end(&self) -> Result<Option<usize>, Errno> {
        self.cursor_to_end_offset(0)
    }

    /// Sets the cursor to the end of the file, plus an offset. Returns the new cursor location.
    ///
    /// Returns [`None`] if cursor operations do not apply to this [`File`]; i.e., the file is a
    /// terminal, socket, pipe, or FIFO.
    ///
    /// Uses the [`lseek`](https://www.man7.org/linux/man-pages/man2/lseek.2.html) Linux syscall
    /// internally.
    ///
    /// # Errors
    ///
    /// This function propagates any errors encountered during the underlying `lseek` operation.
    pub fn cursor_to_end_offset(&self, offset: i64) -> Result<Option<usize>, Errno> {
        self.lseek_wrapper(offset, LseekWhence::SeekEnd)
    }

    /// Wrapper around the `lseek` syscall to reduce code duplication.
    ///
    /// Returns [`None`] if cursor operations do not apply to this [`File`]; i.e., the file is a
    /// terminal, socket, pipe, or FIFO.
    fn lseek_wrapper(&self, offset: i64, whence: LseekWhence) -> Result<Option<usize>, Errno> {
        // SAFETY: The `offset` argument matches the C `off_t` type. The `whence` argument is
        // restricted to the allowed values by the `LseekWhence` enum.
        match unsafe { syscall_result!(SyscallNum::Lseek, self.file_descriptor, offset, whence) } {
            Ok(new_cursor) => Ok(Some(new_cursor)),
            Err(Errno::Espipe) => Ok(None),
            Err(errno) => Err(errno),
        }
    }
}
impl Drop for File {
    fn drop(&mut self) {
        // SAFETY: Statically-chosen arguments. Linux protects against double-closes by gracefully
        // returning EBADF.
        unsafe {
            syscall!(SyscallNum::Close, self.file_descriptor);
        }
    }
}

/// Deletes the file at the given path from the filesystem.
///
/// If other processes still have access to the file, it will remain in existence until the last
/// file descriptor referring to it is closed.
///
/// Internally uses the [`unlink`](https://www.man7.org/linux/man-pages/man2/unlink.2.html) Linux
/// syscall.
///
/// # Errors
///
/// This function propagates any [`Errno`]s returned by the underlying `unlink` syscall.
pub fn rm<NS: Into<NixString>>(path: NS) -> Result<(), Errno> {
    let ns_path: NixString = path.into();

    // SAFETY: The only argument is guaranteed to be null-terminated, valid UTF-8 because of its
    // NixString type.
    unsafe {
        syscall_result!(SyscallNum::Unlink, ns_path.as_ptr())?;
    }
    Ok(())
}

// This is needed to get access to the private file_descriptor field.
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod drop_test {
    use super::*;
    use crate::assert_err;

    const TEST_PATH: &str = "test_files/test.txt";

    #[test_case]
    fn close_file_on_drop() {
        let bad_file_copy: File;
        {
            let file = OpenOptions::new().open(TEST_PATH).unwrap();
            bad_file_copy = File::__new(file.file_descriptor, &OpenOptions::default());
            // file goes out of scope...
        }

        // The file descriptor of the file should now be closed!
        let mut buffer = [0; 64];
        assert_err!(bad_file_copy.read(&mut buffer), Errno::Ebadf);
    }
}
