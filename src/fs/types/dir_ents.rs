//! Module for the info associated with directory entries.

use alloc::string::String;

/// The type of a directory entry.
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum DirEntType {
    /// Unknown file type.
    #[default]
    Unknown = 0,
    /// A named pipe (FIFO).
    Fifo = 1,
    /// A character device.
    Chr = 2,
    /// A directory.
    Dir = 4,
    /// A block device.
    Blk = 6,
    /// A regular file.
    Reg = 8,
    /// A symbolic link.
    Lnk = 10,
    /// A UNIX domain socket.
    Sock = 12,
}
impl From<u8> for DirEntType {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Fifo,
            2 => Self::Chr,
            4 => Self::Dir,
            6 => Self::Blk,
            8 => Self::Reg,
            10 => Self::Lnk,
            12 => Self::Sock,
            _ => Self::Unknown,
        }
    }
}

/// Information about an entry within a directory.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DirEnt {
    /// The type.
    pub d_type: DirEntType,
    /// The name.
    pub name: String,
    /// The [inode](https://en.wikipedia.org/wiki/Inode).
    pub inode: u64,
    /// The raw, C-style header values.
    pub header: DirEntRawHeader,
}
impl DirEnt {
    /// Creates a new [`DirEnt`] from the given raw header and name.
    #[must_use]
    pub fn from_raw(header: DirEntRawHeader, name: String) -> Self {
        Self {
            d_type: header.d_type.into(),
            name,
            inode: header.d_ino,
            header,
        }
    }
}

/// Information about an entry within a directory.
///
/// Corresponds to the `linux_dirent64` datatype described in the
/// [`getdents` manpage](https://man7.org/linux/man-pages/man2/getdents64.2.html).
// It's CRUCIAL this layout is correct! If it isn't, File::dir_ents will be full of UB.
#[repr(C, packed)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(clippy::struct_field_names)]
pub struct DirEntRawHeader {
    /// 64-bit inode number.
    pub d_ino: u64,
    /// Filesystem-specific value with no specific meaning to userspace.
    pub d_off: i64,
    /// Size of this directory entry.
    pub d_reclen: u16,
    /// The type of this directory entry.
    pub d_type: u8,
    // Followed by the directory entry name...
}
