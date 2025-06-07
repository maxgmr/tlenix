//! Module for the stats and information of files.

use crate::{
    Errno, NixString, SyscallNum,
    fs::{FilePermissions, FileType},
    syscall_result,
};

/// Bit mask for the mode bit field.
const MODE_MASK: u32 = 0o7_777;

/// Constant for the "current working directory" file descriptor.
const AT_FDCWD: i32 = -100;

/// Constant for the `statx` system call. If this flag is set, then if the given path name is an
/// empty string or `NULL`, then operate on the file referred to by the given file descriptor.
const AT_EMPTY_PATH: i32 = 0x1000;

/// Constant for the `statx` system call. If this flag is set, then just do whatever `stat` does
/// for file syncing.
const AT_STATX_SYNC_AS_STAT: i32 = 0;

/// Wrapper around the [`statx`](https://man7.org/linux/man-pages/man2/statx.2.html) Linux system
/// call. Gets all the available fields supported by [`FileStatsMask`].
///
/// # Errors
///
/// This function propagates any [`Errno`]s returned by the underlying call to `statx`.
pub(crate) fn statx_get_all<NS: Into<NixString>>(dirfd: i32, path: NS) -> Result<FileStats, Errno> {
    let path_ns: NixString = path.into();
    let flags = AT_EMPTY_PATH | AT_STATX_SYNC_AS_STAT;
    let mask = FileStatsMask::all();
    let mut file_stats_raw = FileStatsRaw::default();

    // SAFETY: The `FileStatsRaw` type is the correct size and alignment for the buffer. The
    // `NixString` type ensures the pointed-to bytes are null-terminated valid UTF-8.
    unsafe {
        syscall_result!(
            SyscallNum::Statx,
            dirfd,
            path_ns.as_ptr(),
            flags,
            mask.bits(),
            &raw mut file_stats_raw
        )?;
    }

    file_stats_raw.try_into()
}

/// Information about a Linux file. Parsed from a [`FileStatsRaw`] returned by the
/// [`statx`](https://man7.org/linux/man-pages/man2/statx.2.html) Linux system call.
///
/// If a field is not included in the [`FileStatsMask`], it will be [`None`], meaning that field
/// was either not requested or not available for the filesystem.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[allow(clippy::struct_field_names)]
pub struct FileStats {
    /// The file fields retrieved by the underlying `statx` call.
    pub file_stats_mask: FileStatsMask,
    /// The [`FileType`] of the file.
    pub file_type: Option<FileType>,
    /// The [`FilePermissions`]/operating mode of the file.
    pub mode: Option<FilePermissions>,
    /// The "preferred" block size for efficient filesystem I/O.
    pub block_size: Option<u32>,
    /// Further status information about the file.
    pub attributes: FileAttributes,
    /// The number of hard links on a file.
    pub hard_links: Option<u32>,
    /// The user ID of the file owner.
    pub uid: Option<u32>,
    /// The ID of the group owner of the file.
    pub gid: Option<u32>,
    /// The inode number of the file.
    pub inode: Option<u64>,
    /// The size of the file in bytes.
    pub size: Option<u64>,
    /// The number of 512-byte blocks allocated to the file.
    pub blocks: Option<u64>,
    /// Last access time of the file.
    pub access_time: Option<FileTimestamp>,
    /// Creation time of the file.
    pub creation_time: Option<FileTimestamp>,
    /// Last status change time of the file.
    pub status_change_time: Option<FileTimestamp>,
    /// Last modification time of the file.
    pub modification_time: Option<FileTimestamp>,
    /// Major ID of the device containing this file.
    pub major_device_id: Option<u32>,
    /// Minor ID of the device containing this file.
    pub minor_device_id: Option<u32>,
    /// Mount ID of the mount containing the file.
    pub mount_id: Option<u64>,
    /// Alignment (in bytes) required for user memory buffers for direct I/O on this file, or 0 if
    /// direct I/O is not supported on this file.
    pub direct_io_memory_alignment: Option<u32>,
    /// Alignment (in bytes) required for file offsets and I/O segment lengths for direct I/O on
    /// this file, or 0 if direct I/O is not supported on this file.
    pub direct_io_offset_alignment: Option<u32>,
    /// Subvolume number of the current file.
    pub subvolume: Option<u64>,
    /// The minimum size (in bytes) supported for direct I/O on the file to be written with
    /// torn-write protection.
    pub atomic_write_unit_min: Option<u32>,
    /// The maximum size (in bytes) supported for direct I/O on the file to be written with
    /// torn-write protection.
    pub atomic_write_unit_max: Option<u32>,
    /// The maximum number of elements in an array of vectors for a write with torn-write
    /// protection enabled.
    pub atomic_write_segments_max: Option<u32>,
    /// File offset alignment for direct I/O reads.
    pub direct_io_read_offset_alignment: Option<u32>,
}
impl FileStats {
    /// Gets information about a file located at the given path.
    ///
    /// Internally uses the [`statx`](https://man7.org/linux/man-pages/man2/statx.2.html) Linux
    /// system call.
    ///
    /// # Errors
    ///
    /// This function propagates any [`Errno`]s returned from the underlying call to `statx`.
    pub fn try_from_path<NS: Into<NixString>>(path: NS) -> Result<Self, Errno> {
        statx_get_all(AT_FDCWD, path)
    }

    fn masked_stat<T>(stat: T, flag: FileStatsMask, mask: FileStatsMask) -> Option<T> {
        if mask.intersects(flag) {
            Some(stat)
        } else {
            None
        }
    }
}
impl TryFrom<FileStatsRaw> for FileStats {
    type Error = Errno;
    fn try_from(value: FileStatsRaw) -> Result<Self, Self::Error> {
        let file_stats_mask = FileStatsMask::from_bits_truncate(value.mask);
        let file_type: Option<FileType> =
            match Self::masked_stat(value.mode, FileStatsMask::TYPE, file_stats_mask) {
                Some(mode) => Some(u32::from(mode).try_into()?),
                None => None,
            };
        let mode =
            Self::masked_stat(value.mode, FileStatsMask::MODE, file_stats_mask).map(|mode| {
                FilePermissions::from_bits_truncate((u32::from(mode) & MODE_MASK) as usize)
            });
        let block_size = Self::masked_stat(value.blksize, FileStatsMask::BLOCKS, file_stats_mask);
        let attributes = FileAttributes {
            flags: FileAttributeFlags::from_bits_truncate(value.attributes),
            mask: FileAttributeFlags::from_bits_truncate(value.attributes_mask),
        };
        let hard_links = Self::masked_stat(value.nlink, FileStatsMask::NLINK, file_stats_mask);
        let uid = Self::masked_stat(value.uid, FileStatsMask::UID, file_stats_mask);
        let gid = Self::masked_stat(value.gid, FileStatsMask::GID, file_stats_mask);
        let inode = Self::masked_stat(value.ino, FileStatsMask::INO, file_stats_mask);
        let size = Self::masked_stat(value.size, FileStatsMask::SIZE, file_stats_mask);
        let blocks = Self::masked_stat(value.blocks, FileStatsMask::BLOCKS, file_stats_mask);
        let access_time = Self::masked_stat(value.atime, FileStatsMask::ATIME, file_stats_mask);
        let creation_time = Self::masked_stat(value.btime, FileStatsMask::BTIME, file_stats_mask);
        let status_change_time =
            Self::masked_stat(value.ctime, FileStatsMask::CTIME, file_stats_mask);
        let modification_time =
            Self::masked_stat(value.mtime, FileStatsMask::MTIME, file_stats_mask);
        let major_device_id = Self::masked_stat(
            value.rdev_major,
            FileStatsMask::MNT_ID_UNIQUE,
            file_stats_mask,
        );
        let minor_device_id = Self::masked_stat(
            value.rdev_minor,
            FileStatsMask::MNT_ID_UNIQUE,
            file_stats_mask,
        );
        let mount_id = Self::masked_stat(value.mnt_id, FileStatsMask::MNT_ID, file_stats_mask);
        let direct_io_memory_alignment = Self::masked_stat(
            value.dio_mem_align,
            FileStatsMask::DIOALIGN,
            file_stats_mask,
        );
        let direct_io_offset_alignment = Self::masked_stat(
            value.dio_offset_align,
            FileStatsMask::DIOALIGN,
            file_stats_mask,
        );
        let subvolume = Self::masked_stat(value.subvol, FileStatsMask::SUBVOL, file_stats_mask);
        let atomic_write_unit_min = Self::masked_stat(
            value.atomic_write_unit_min,
            FileStatsMask::WRITE_ATOMIC,
            file_stats_mask,
        );
        let atomic_write_unit_max = Self::masked_stat(
            value.atomic_write_unit_max,
            FileStatsMask::WRITE_ATOMIC,
            file_stats_mask,
        );
        let atomic_write_segments_max = Self::masked_stat(
            value.atomic_write_segments_max,
            FileStatsMask::WRITE_ATOMIC,
            file_stats_mask,
        );
        let direct_io_read_offset_alignment = Self::masked_stat(
            value.dio_read_offset_align,
            FileStatsMask::DIO_READ_ALIGN,
            file_stats_mask,
        );
        Ok(Self {
            file_stats_mask,
            file_type,
            mode,
            block_size,
            attributes,
            hard_links,
            uid,
            gid,
            inode,
            size,
            blocks,
            access_time,
            creation_time,
            status_change_time,
            modification_time,
            major_device_id,
            minor_device_id,
            mount_id,
            direct_io_memory_alignment,
            direct_io_offset_alignment,
            subvolume,
            atomic_write_unit_min,
            atomic_write_unit_max,
            atomic_write_segments_max,
            direct_io_read_offset_alignment,
        })
    }
}

/// Information about a Linux file. Directly corresponds to the
/// [`statx`](https://man7.org/linux/man-pages/man2/statx.2.html) struct in `libc`.
#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct FileStatsRaw {
    /// Mask of bits indicating filled fields.
    pub mask: u32,
    /// Block size of filesystem I/O.
    pub blksize: u32,
    /// Extra file attribute indicators.
    pub attributes: u64,
    /// Number of hard links.
    pub nlink: u32,
    /// User ID of owner.
    pub uid: u32,
    /// Group ID of owner.
    pub gid: u32,
    /// File type and mode.
    pub mode: u16,
    /// Padding.
    __pad1: [u16; 1],
    /// Inode number.
    pub ino: u64,
    /// Total size in bytes.
    pub size: u64,
    /// Number of 512B blocks allocated.
    pub blocks: u64,
    /// Mask to show what's supported in `stx_attributes`.
    pub attributes_mask: u64,
    /// Timestamp of last access.
    pub atime: FileTimestamp,
    /// Timestamp of creation.
    pub btime: FileTimestamp,
    /// Timestamp of last status change.
    pub ctime: FileTimestamp,
    /// Timestamp of last modification.
    pub mtime: FileTimestamp,
    /// If this file is a device, this field contains the major ID of the device.
    pub rdev_major: u32,
    /// If this file is a device, this field contains the minor ID of the device.
    pub rdev_minor: u32,
    /// Mount ID.
    pub mnt_id: u64,
    /// Direct I/O memory restriction alignment.
    pub dio_mem_align: u32,
    /// Direct I/O memory restriction offset.
    pub dio_offset_align: u32,
    /// Subvolume identifier.
    pub subvol: u64,
    /// Direct I/O unit minimum atomic write limit.
    pub atomic_write_unit_min: u32,
    /// Direct I/O unit maximum atomic write limit.
    pub atomic_write_unit_max: u32,
    /// Direct I/O maximum segment atomic write limit.
    pub atomic_write_segments_max: u32,
    /// File offset alignment for direct I/O reads.
    pub dio_read_offset_align: u32,
    /// Padding.
    _pad3: [u64; 12],
}

/// Macro to impl the different fns retrieving the different [`FileAttributes`] values.
macro_rules! file_attributes_getters {
    {
        $(
            $(#[$when_true:meta])*
            $method_name:ident($flag:ident);
        )*
    } => {
        $(
            /// This function returns `true` when the file
            $(#[$when_true])*
            ///
            /// If this flag is not supported, then this function returns `None`.
            #[must_use]
            pub fn $method_name(&self) -> Option<bool> {
                if self.mask.intersects(FileAttributeFlags::$flag) {
                    Some(self.flags.intersects(FileAttributeFlags::$flag))
                } else {
                    None
                }
            }
        )*
    };
}

/// Additional attributes of a Linux file. Parsed from a file's attributes and attributes mask.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FileAttributes {
    flags: FileAttributeFlags,
    mask: FileAttributeFlags,
}
impl FileAttributes {
    file_attributes_getters! {
        /// is compressed by the filesystem and may take extra resources to access.
        is_compressed(COMPRESSED);
        /// cannot be modified/deleted/renamed/hard-linked/written to.
        is_immutable(IMMUTABLE);
        /// can only be opened in append mode for writing.
        is_append(APPEND);
        /// not a candidate for backup when a backup program is run.
        is_nodump(NODUMP);
        /// required to be encrypted with a key.
        is_encrypted(ENCRYPTED);
        /// cannot be written to, and all reads from it will be verified against a cryptographic
        /// hash that covers the entire file.
        verity(VERITY);
        /// supports torn-write protection.
        write_atomic(WRITE_ATOMIC);
        /// is in the DAX (CPU direct access) state.
        dax(DAX);
        /// is the root of the mount.
        mount_root(MOUNT_ROOT);

    }
}

bitflags::bitflags! {
    /// Additional attributes of a Linux file. Parsed from [`FileStatsRaw::attributes`]. These are
    /// the flags themselves. Only when used in combination of the provided mask do they have
    /// meaning.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct FileAttributeFlags: u64 {
        /// The file is compressed by the filesystem.
        const COMPRESSED = 0x4;
        /// The file cannot be modified, deleted, hard-linked, or renamed.
        const IMMUTABLE = 0x10;
        /// The file can only be opened in append mode for writing.
        const APPEND = 0x20;
        /// The file is not a backup candidate.
        const NODUMP = 0x40;
        /// The file must be encrypted with a key.
        const ENCRYPTED = 0x800;
        /// The file cannot be written to, and all reads from it will be verified against a
        /// cryptographic hash that covers the entire file.
        const VERITY = 0x10_0000;
        /// The file supports torn-write protection.
        const WRITE_ATOMIC = 0x40_0000;
        /// The file is in the CPU direct access state, minimizing software cache efforts for both
        /// I/O and memory mappings of this file.
        const DAX = 0x20_0000;
        /// The file is the root of a mount.
        const MOUNT_ROOT = 0x2000;
    }
}

bitflags::bitflags! {
    /// Query result/request mask for the
    /// [`statx`](https://man7.org/linux/man-pages/man2/statx.2.html) Linux system call.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct FileStatsMask: u32 {
        /// Want/got [`FileStatsRaw::mode`] & [`S_IFMT`]
        const TYPE = 0x1;
        /// Want/got [`FileStatsRaw::mode`] & `!`[`S_IFMT`]
        const MODE = 0x2;
        /// Want/got [`FileStatsRaw::nlink`]
        const NLINK = 0x4;
        /// Want/got [`FileStatsRaw::uid`]
        const UID = 0x8;
        /// Want/got [`FileStatsRaw::gid`]
        const GID = 0x10;
        /// Want/got [`FileStatsRaw::atime`]
        const ATIME = 0x20;
        /// Want/got [`FileStatsRaw::mtime`]
        const MTIME = 0x40;
        /// Want/got [`FileStatsRaw::ctime`]
        const CTIME = 0x80;
        /// Want/got [`FileStatsRaw::ino`]
        const INO = 0x100;
        /// Want/got [`FileStatsRaw::size`]
        const SIZE = 0x200;
        /// Want/got [`FileStatsRaw::blocks`]
        const BLOCKS = 0x400;
        /// All of the above fields (in the normal stat struct)
        const BASIC_STATS = 0x7ff;
        /// Want/got [`FileStatsRaw::btime`]
        const BTIME = 0x800;
        /// Want/got [`FileStatsRaw::mnt_id`]
        const MNT_ID = 0x1000;
        /// Want/got [`FileStatsRaw::dio_mem_align`] and [`FileStatsRaw::dio_offset_align`].
        const DIOALIGN = 0x2000;
        /// Want/got unique/extended [`FileStatsRaw::mnt_id`]
        const MNT_ID_UNIQUE = 0x4000;
        /// Want/got [`FileStatsRaw::subvol`]
        const SUBVOL= 0x8000;
        /// Want/got [`FileStatsRaw::atomic_write_unit_min`],
        /// [`FileStatsRaw::atomic_write_unit_max`], and
        /// [`FileStatsRaw::atomic_write_segments_max`]
        const WRITE_ATOMIC = 0x1_0000;
        /// Want/got [`FileStatsRaw::dio_read_offset_align`].
        const DIO_READ_ALIGN = 0x2_0000;
    }
}

/// A file timestamp. Directly corresponds to the
/// [`statx_timestamp`](https://man7.org/linux/man-pages/man2/statx.2.html) type in `libc`.
#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct FileTimestamp {
    /// Seconds since the epoch (UNIX time)
    pub sec: i64,
    /// Nanoseconds since [`FileStatsTimestampRaw::sec`]
    pub nsec: u32,
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
