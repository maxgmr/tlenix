//! This module is responsible for the [`File`] type and all associated file operations.

use crate::{Errno, SyscallArg, SyscallNum, fs::OpenOptions, syscall_result};

/// Bit mask for the file type bit field.
const S_IFMT: u32 = 0o0_170_000;

/// All possible values which can be sent to the `lseek` syscall to declare its functionality.
#[repr(usize)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(clippy::enum_variant_names)]
#[allow(dead_code)]
enum LseekWhence {
    SeekSet,
    SeekCur,
    SeekEnd,
    SeekData,
    SeekHole,
}
impl From<LseekWhence> for SyscallArg {
    fn from(value: LseekWhence) -> Self {
        Self::from(value as usize)
    }
}

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

/// The type of a given [`File`].
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FileType {
    /// A [Unix domain socket](https://en.wikipedia.org/wiki/Unix_domain_socket).
    Socket = 0o0_140_000,
    /// A [symbolic link](https://en.wikipedia.org/wiki/Symbolic_link).
    SymbolicLink = 0o0_120_000,
    /// A regular file.
    RegularFile = 0o0_100_000,
    /// A [block device file](https://en.wikipedia.org/wiki/Device_file#Block_devices).
    BlockDevice = 0o0_060_000,
    /// A file directory.
    Directory = 0o0_040_000,
    /// A [character device file](https://en.wikipedia.org/wiki/Device_file#Character_devices).
    CharacterDevice = 0o0_020_000,
    /// A first-in-first-out [named pipe](https://en.wikipedia.org/wiki/Named_pipe).
    Fifo = 0o0_010_000,
}
impl TryFrom<u32> for FileType {
    type Error = Errno;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let masked_value = value & S_IFMT;

        if masked_value == (Self::Socket as u32) {
            Ok(Self::Socket)
        } else if masked_value == (Self::SymbolicLink as u32) {
            Ok(Self::SymbolicLink)
        } else if masked_value == (Self::RegularFile as u32) {
            Ok(Self::RegularFile)
        } else if masked_value == (Self::BlockDevice as u32) {
            Ok(Self::BlockDevice)
        } else if masked_value == (Self::Directory as u32) {
            Ok(Self::Directory)
        } else if masked_value == (Self::CharacterDevice as u32) {
            Ok(Self::CharacterDevice)
        } else if masked_value == (Self::Fifo as u32) {
            Ok(Self::Fifo)
        } else {
            Err(Errno::Eio)
        }
    }
}

/// Information about a given [`File`]. Calculated from a [`FileStatRaw`].
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileStat {
    /// The raw file stats.
    pub file_stat_raw: FileStatRaw,
    /// The type of the file.
    pub file_type: FileType,
}
impl TryFrom<FileStatRaw> for FileStat {
    type Error = Errno;
    fn try_from(value: FileStatRaw) -> Result<Self, Self::Error> {
        let file_type = value.st_mode.try_into()?;
        Ok(Self {
            file_stat_raw: value,
            file_type,
        })
    }
}

/// Information about a given [`File`]. Corresponds to the
/// [`stat`](https://man7.org/linux/man-pages/man3/stat.3type.html) struct in `libc`.
#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct FileStatRaw {
    /// The device on which this file resides.
    pub st_dev: u64,
    /// The file's inode number.
    pub st_ino: u64,
    /// The number of hard links to the file.
    pub st_nlink: u64,
    /// The file type and mode.
    pub st_mode: u32,
    /// The user ID of the file owner.
    pub st_uid: u32,
    /// The group ID of the file owner.
    pub st_gid: u32,
    /// Padding.
    __pad0: i32,
    /// The device that this file represents.
    pub st_rdev: u64,
    /// The size of the file in bytes.
    pub st_size: i64,
    /// The "preferred" block size for efficient filesystem I/O.
    pub st_blksize: i64,
    /// The number of blocks allocated to the file, in 512-byte units.
    pub st_blocks: i64,
    /// The time of the last access of file data.
    pub st_atime: i64,
    /// The time of the last access of file data in nanoseconds.
    pub st_atime_nsec: i64,
    /// The time of the last modification of file data.
    pub st_mtime: i64,
    /// The time of the last modification of file data in nanoseconds.
    pub st_mtime_nsec: i64,
    /// The time of the last status change.
    pub st_ctime: i64,
    /// The time of the last status change in nanoseconds.
    pub st_ctime_nsec: i64,
    /// Unused space.
    __unused: [i64; 3],
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

    /// Gets the current cursor location within the [`File`].
    ///
    /// Uses the [`lseek`](https://www.man7.org/linux/man-pages/man2/lseek.2.html) Linux syscall
    /// internally.
    ///
    /// # Errors
    ///
    /// This function propagates any errors encountered during the underlying `lseek` operation.
    pub fn cursor(&self) -> Result<usize, Errno> {
        // SAFETY: The arguments are correct and statically-determined.
        unsafe {
            syscall_result!(
                SyscallNum::Lseek,
                self.file_descriptor,
                0,
                LseekWhence::SeekCur
            )
        }
    }
}
