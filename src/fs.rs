//! Filesystem. Responsible for file operations.

use bitflags::bitflags;

use crate::{Errno, SyscallNum, data::NullTermStr, syscall_result};

bitflags! {
    /// All the different flags which can be set for the [open](https://www.man7.org/linux/man-pages/man2/open.2.html)
    /// Linux syscall.
    pub struct OpenFlags: usize {
        /// File open flag: Open file read-only.
        const O_RDONLY = 0x0;
        /// File open flag: Open file write-only.
        const O_WRONLY= 0x1;
        /// File open flag: Open file read/write.
        const O_RDWR= 0x2;
        /// File open flag: If `path` does not exist, create as regular file.
        const O_CREAT= 0x40;
        /// File open flag: Ensure that this call creates the file. Throw error if file
        /// already exists.
        const O_EXCL= 0x80;
        /// File open flag: If `path` refers to a terminal device, it won't become the process's
        /// controlling terminal.
        const O_NOCTTY= 0x100;
        /// File open flag: If the file already exists and the access mode allows writing, it will
        /// be truncated to length 0.
        const O_TRUNC= 0x200;
        /// File open flag: Open in append mode.
        const O_APPEND= 0x400;
        /// File open flag: Open in nonblocking mode when possible.
        const O_NONBLOCK= 0x800;
        /// File open flag: Open in nonblocking mode when possible.
        const O_NDELAY= 0x800;
        /// File open flag: Write operations on the file will complete according to synchronised
        /// I/O data integrity completion.
        const O_DSYNC= 0x1000;
        /// File open flag: Enable signal-drive I/O.
        const O_ASYNC= 0x2000;
        /// File open flag: Minimise cache effects of the I/O to and from this file.
        const O_DIRECT= 0x4000;
        /// File open flag: If `path` is not a directory, cause the open to fail.
        const O_DIRECTORY= 0x1_0000;
        /// File open flag: Fail if the trailing component of `path` is a symlink.
        const O_NOFOLLOW= 0x2_0000;
        /// File open flag: Don't update the file last access time when the file is read.
        const O_NOATIME= 0x4_0000;
        /// File open flag: Enable close-on-exec for new file descriptor.
        const O_CLOEXEC= 0x8_0000;
        /// File open flag: Write operations on the file will complete according to synchronised
        /// I/O file integrity completion.
        const O_SYNC= 0x10_1000;
        /// File open flag: Obtain a file descriptor without opening the file.
        const O_PATH = 0x20_0000;
    }
}

/// Process-unique identifier for a file or other input/output resource.
/// [Wikipedia](https://en.wikipedia.org/wiki/File_descriptor)
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileDescriptor(usize);

/// Read a single byte from a file at the given [`FileDescriptor`].
///
/// Will return [`None`] if the end of file has been reached.
///
/// # Errors
///
/// Will propagate any error if the underlying [read](https://www.man7.org/linux/man-pages/man2/read.2.html)
/// syscall fails.
pub fn read_byte(file_descriptor: FileDescriptor) -> Result<Option<u8>, Errno> {
    let mut byte: u8 = 0x00;

    let bytes_read = unsafe {
        syscall_result!(
            SyscallNum::Read,
            file_descriptor.0,
            &raw mut byte as usize,
            1
        )?
    };

    if bytes_read == 0 {
        return Ok(None);
    }

    Ok(Some(byte))
}

/// Writes a single byte to the file at the given [`FileDescriptor`]. Returns the number of bytes
/// written.
///
/// # Errors
///
/// Can return any errors associated with the [write](https://www.man7.org/linux/man-pages/man2/write.2.html) Linux syscall.
pub fn write_byte(file_descriptor: FileDescriptor, byte: u8) -> Result<usize, Errno> {
    // SAFETY: The pointer to the byte is valid. The buffer size is statically-chosen and matches
    // the single byte being written. Any issues with user-given arguments are handled gracefully
    // by the underlying syscall.
    let bytes_written = unsafe {
        syscall_result!(
            SyscallNum::Write,
            file_descriptor.0,
            &raw const byte as usize,
            1
        )?
    };

    Ok(bytes_written)
}

/// Read from a file into a buffer.
///
/// # Errors
///
/// Will return [`Err`] if `path` does not exist or if the user doesn't have permission to read the
/// given file.
pub fn read_from_file<const PATHN: usize, const BUFN: usize>(
    path: &NullTermStr<PATHN>,
) -> Result<[u8; BUFN], Errno> {
    let file_descriptor = open_no_create::<PATHN>(path, &OpenFlags::O_RDONLY)?;
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

/// Opens a file at the given path, returning its given [`FileDescriptor`].
///
/// Wrapper around the [open](https://www.man7.org/linux/man-pages/man2/open.2.html) Linux syscall.
///
/// To ensure safety, this function cannot be used to create a file or a tempfile. This is because
/// this function does not provide a `mode` syscall argument.
///
/// `O_RDONLY | O_TRUNC` is undefined behaviour and also gets rejected.
///
/// # Errors
///
/// This function returns an [`Errno`] if:
///
/// 1. [`OpenFlags::O_CREAT`] or [`OpenFlags::O_TMPFILE`] flags are set. In this case,
///    [`Errno::Eperm`] is returned.
/// 2. [`OpenFlags::O_RDONLY`] and [`OpenFlags::O_TRUNC`] flags are set. In this case,
///    [`Errno::Eperm`] is returned.
/// 3. The underlying [open](https://www.man7.org/linux/man-pages/man2/open.2.html) syscall returns
///    an [`Errno`].
pub fn open_no_create<const N: usize>(
    path: &NullTermStr<N>,
    flags: &OpenFlags,
) -> Result<FileDescriptor, Errno> {
    if flags.intersects(OpenFlags::O_CREAT) {
        // No file creation allowed!
        return Err(Errno::Eperm);
    }
    if flags.contains(OpenFlags::O_RDONLY | OpenFlags::O_TRUNC) {
        // Using these two flags together is UB!
        return Err(Errno::Eperm);
    }

    let raw_fd =
        unsafe { syscall_result!(SyscallNum::Open, path.as_ptr() as usize, flags.bits())? };
    Ok(FileDescriptor(raw_fd))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{eprintln, nulltermstr, print, println};

    const TEST_PATH: NullTermStr<20> = nulltermstr!(b"test_files/test.txt");
    const TEST_PATH_CONTENTS: [u8; 68] =
        *b"Hello! I hope you can read me without any issues! - Max (\xE9\xA9\xAC\xE5\x85\x8B\xE6\x96\xAF)\n";

    #[test_case]
    fn read_file() {
        let result_bytes: [u8; 256] = read_from_file(&TEST_PATH)
            .inspect_err(|e| eprintln!("{}", e.as_str()))
            .unwrap();
        assert_eq!(&result_bytes[..68], TEST_PATH_CONTENTS);
    }

    #[test_case]
    fn read_byte() {
        let test_file = open_no_create(&TEST_PATH, &OpenFlags::O_RDONLY).unwrap();
        for &expected_byte in &TEST_PATH_CONTENTS {
            let byte = super::read_byte(test_file).unwrap().unwrap();
            print!("{}", str::from_utf8(&[byte]).unwrap_or("�"));
            assert_eq!(byte, expected_byte);
        }
        println!();

        match super::read_byte(test_file).unwrap() {
            None => (), // OK!
            Some(byte) => {
                println!("BAD BYTE: '{}'", str::from_utf8(&[byte]).unwrap_or("�"));
                panic!("Read too many bytes!");
            }
        }
    }

    #[test_case]
    fn path_dne() {
        let bad_path = NullTermStr::<17>::from(*b"/not/a/real/path");
        match read_from_file::<17, 16>(&bad_path) {
            Err(Errno::Enoent) => (), // OK!
            _ => panic!("expected Err(Errno::Enoent)"),
        }
    }

    #[test_case]
    fn no_creat() {
        let dummy_path = NullTermStr::<2>::from(*b"/");
        match open_no_create(&dummy_path, &OpenFlags::O_CREAT) {
            Err(Errno::Eperm) => (), // OK!
            _ => panic!("expected Err(Errno::Eperm)"),
        }
    }

    #[test_case]
    fn no_rdonly_trunc() {
        let dummy_path = NullTermStr::<2>::from(*b"/");
        match open_no_create(&dummy_path, &(OpenFlags::O_RDONLY | OpenFlags::O_TRUNC)) {
            Err(Errno::Eperm) => (), // OK!
            _ => panic!("expected Err(Errno::Eperm)"),
        }
    }
}
