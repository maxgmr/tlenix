//! Filesystem. Responsible for file operations.

use crate::consts::O_RDONLY;

use super::{Errno, SyscallNum, data::NullTermStr, syscall_result};

/// Process-unique identifier for a file or other input/output resource.
/// [Wikipedia](https://en.wikipedia.org/wiki/File_descriptor)
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileDescriptor(usize);

/// Read from a file into a buffer.
///
/// # Errors
///
/// Will return [`Err`] if `path` does not exist or if the user doesn't have permission to read the
/// given file.
pub fn read_from_file<const PATHN: usize, const BUFN: usize>(
    path: &NullTermStr<PATHN>,
) -> Result<[u8; BUFN], Errno> {
    let file_descriptor = open_ro(path)?;
    let mut buf = [0x00_u8; BUFN];
    // SAFETY: The arguments are correct and the buffer length is derived directly from the buffer
    // itself.
    unsafe {
        syscall_result!(
            SyscallNum::Read,
            file_descriptor.0,
            buf.as_mut_ptr() as usize,
            buf.len()
        )?;
    };

    Ok(buf)
}

/// Wrapper around [open](https://www.man7.org/linux/man-pages/man2/read.2.html) Linux syscall.
///
/// Open a file as readonly. Return [`Errno`] if the file does not exist.
fn open_ro<const N: usize>(path: &NullTermStr<N>) -> Result<FileDescriptor, Errno> {
    // SAFETY: Any errors are caught and returned as `Errno`. The args are correct. `as_ptr()`
    // returns a `u8` which always fits into `usize`.
    unsafe {
        syscall_result!(SyscallNum::Open, path.as_ptr() as usize, O_RDONLY).map(FileDescriptor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eprintln;

    const TEST_PATH: [u8; 19] = *b"test_files/test.txt";
    const TEST_PATH_CONTENTS: [u8; 67] =
        *b"Hello! I hope you can read me without any issues! - Max (\xE9\xA9\xAC\xE5\x85\x8B\xE6\x96\xAF)";

    #[test_case]
    fn read_file() {
        let result_bytes: [u8; 256] = read_from_file::<20, 256>(&NullTermStr::from(TEST_PATH))
            .inspect_err(|e| eprintln!("{}", e.as_str()))
            .unwrap();
        assert_eq!(&result_bytes[..67], TEST_PATH_CONTENTS);
    }
}
