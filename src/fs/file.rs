//! This module is responsible for the [`File`] type and all associated file operations.

use crate::{Errno, SyscallNum, fs::OpenOptions, syscall_result};

/// Process-unique identifier for a file or other input/output resource.
/// [Wikipedia](https://en.wikipedia.org/wiki/File_descriptor)
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileDescriptor(usize);
impl From<usize> for FileDescriptor {
    fn from(value: usize) -> Self {
        Self(value)
    }
}
impl From<FileDescriptor> for usize {
    fn from(value: FileDescriptor) -> Self {
        value.0
    }
}

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
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;

    const THIS_PATH: &str = "src/fs/file.rs";
    const TEST_PATH: &str = "test_files/test.txt";
    const TEST_PATH_CONTENTS: &str =
        "Hello! I hope you can read me without any issues! - Max (马克斯)";

    #[test_case]
    fn read_bytes() {
        const EXPECTED_STR: &str = "//! This module is";
        let expected = EXPECTED_STR.as_bytes();

        let mut buffer = [0; EXPECTED_STR.len()];
        let bytes_read = OpenOptions::new()
            .open(THIS_PATH)
            .unwrap()
            .read(&mut buffer)
            .unwrap();

        assert_eq!(bytes_read, EXPECTED_STR.len());
        assert_eq!(expected, buffer);
    }

    #[test_case]
    fn read_utf8() {
        let mut buffer = [0; TEST_PATH_CONTENTS.len()];
        let bytes_read = OpenOptions::new()
            .read_write()
            .open(TEST_PATH)
            .unwrap()
            .read(&mut buffer)
            .unwrap();

        assert_eq!(bytes_read, TEST_PATH_CONTENTS.len());
        assert_eq!(TEST_PATH_CONTENTS, str::from_utf8(&buffer).unwrap());
    }

    #[test_case]
    fn read_past_end() {
        let mut buffer = [0; TEST_PATH_CONTENTS.len() - 1];
        let file = OpenOptions::new().open(TEST_PATH).unwrap();
        let bytes_read = file.read(&mut buffer).unwrap();
        let expected = &TEST_PATH_CONTENTS.as_bytes()[..TEST_PATH_CONTENTS.len() - 1];
        assert_eq!(bytes_read, buffer.len());
        assert_eq!(buffer, expected);

        // Attempt to read past the end
        let bytes_read = file.read(&mut buffer).unwrap();
        let mut expected_2 = [0; TEST_PATH_CONTENTS.len() - 1];
        expected_2.copy_from_slice(expected);
        expected_2[0] = TEST_PATH_CONTENTS.as_bytes()[TEST_PATH_CONTENTS.len() - 1];
        expected_2[1] = b'\n';
        assert_eq!(bytes_read, 2);
        assert_eq!(buffer, expected_2);

        let bytes_read = file.read(&mut buffer).unwrap();
        assert_eq!(bytes_read, 0);
        assert_eq!(buffer, expected_2);
    }

    #[test_case]
    fn read_wo() {
        let mut buffer = [0; 1];
        match OpenOptions::new()
            .write_only()
            .open(TEST_PATH)
            .unwrap()
            .read(&mut buffer)
        {
            Err(Errno::Ebadf) => {} // OK!
            val => panic!("expected Err(Errno::Ebadf), got {:?}", val),
        }
    }

    #[test_case]
    fn read_dir() {
        let mut buffer = [0; 1];
        match OpenOptions::new().open("/").unwrap().read(&mut buffer) {
            Err(Errno::Eisdir) => {} // OK!
            val => panic!("expected Err(Errno::Eisdir), got {val:?}"),
        }
    }

    #[test_case]
    fn write_ro() {
        let buffer = *b"irrelevant";
        match OpenOptions::new().open(TEST_PATH).unwrap().write(&buffer) {
            Err(Errno::Ebadf) => {} // OK!
            val => panic!("expected Err(Errno::Ebadf), got {val:?}"),
        }
    }
}
