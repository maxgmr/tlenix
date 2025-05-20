//! Various types useful for filesystem functionality.

use crate::{Errno, SyscallArg};

/// Bit mask for the file type bit field.
const S_IFMT: u32 = 0o0_170_000;

/// All possible values which can be sent to the `lseek` syscall to declare its functionality.
#[repr(usize)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(clippy::enum_variant_names)]
pub enum LseekWhence {
    /// The file offset is set to `offset` bytes.
    SeekSet,
    /// The file offset is set to its current location plus `offset` bytes.
    SeekCur,
    /// The file offset is set to the size of the file plus `offset` bytes.
    SeekEnd,
    /// Adjust the file offset to the next location in the file which contains data.
    SeekData,
    /// Adjust the file offset to the next hole in the file. If no holes, the offset is set to the
    /// end of the file.
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

/// The type of a given [`crate::fs::File`].
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

/// Information about a given [`crate::fs::File`]. Calculated from a [`FileStatRaw`].
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

/// Information about a given [`crate::fs::File`]. Corresponds to the
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
