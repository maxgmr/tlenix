//! Functionality related to mounting and unmounting filesystems.

use core::ptr;

use crate::{Errno, SyscallNum, nix_str::NixString, syscall_result};

/// A list of possible Linux filesystem types.
///
/// This list is not exhaustive and may grow in the future.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum FilesystemType {
    /// Process info.
    Proc,
    /// Kernel and device info.
    Sysfs,
    /// Temporary file storage in volatile memory.
    Tmpfs,
    /// Automated device notes populated by the kernel.
    Devtmpfs,
    /// RAM-based debugging.
    Debugfs,
    /// Interface for security info.
    Securityfs,
    /// RAM-based filesystem.
    Ramfs,
    /// Kernel-based automount utility.
    Autofs,
    /// Journaling file system.
    Ext4,
    /// Journaling file system.
    Xfs,
    /// EFI/FAT32 file system.
    Vfat,
    /// Network file system.
    Nfs,
    /// Pseudoterminals.
    Devpts,
    /// Huge pages.
    Hugetlbfs,
    /// Message queue.
    Mqueue,
}
impl From<FilesystemType> for NixString {
    fn from(value: FilesystemType) -> Self {
        (match value {
            FilesystemType::Proc => "proc",
            FilesystemType::Sysfs => "sysfs",
            FilesystemType::Tmpfs => "tmpfs",
            FilesystemType::Devtmpfs => "devtmpfs",
            FilesystemType::Debugfs => "debugfs",
            FilesystemType::Securityfs => "securityfs",
            FilesystemType::Ramfs => "ramfs",
            FilesystemType::Autofs => "autofs",
            FilesystemType::Ext4 => "ext4",
            FilesystemType::Xfs => "xfs",
            FilesystemType::Vfat => "vfat",
            FilesystemType::Nfs => "nfs",
            FilesystemType::Devpts => "devpts",
            FilesystemType::Hugetlbfs => "hugetlbfs",
            FilesystemType::Mqueue => "mqueue",
        })
        .into()
    }
}

bitflags::bitflags! {
    /// All the different flags which can be sent to the [`mount`] function.
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct MountFlags: u64 {
        /// Remount an existing mount.
        const MS_REMOUNT = 0x20;
        /// Create a bind mount.
        const MS_BIND = 0x1_000;
        /// Set shared propagation type.
        const MS_SHARED = 0x100_000;
        /// Set private propagation type.
        const MS_PRIVATE = 0x40_000;
        /// Set slave propagation type.
        const MS_SLAVE = 0x80_000;
        /// Set unbindable propagation type.
        const MS_UNBINDABLE = 0x20_000;
        /// Move an existing mount to a new location.
        const MS_MOVE = 0x2_000;
        /// Make directory changes on this filesystem synchronous.
        const MS_DIRSYNC = 0x80;
        /// Reduce on-disk updates of inode timestamps.
        const MS_LAZYTIME = 0x2_000_000;
        /// Permit mandatory locking on files in this filesystem.
        const MS_MANDLOCK = 0x40;
        /// Do not update access times for files on this filesystem.
        const MS_NOATIME = 0x400;
        /// Do not allow access to devices on this filesystem.
        const MS_NODEV = 0x4;
        /// Do not update access times for directories on this filesystem.
        const MS_NODIRATIME = 0x800;
        /// Do not allow programs to be executed from this filesystem.
        const MS_NOEXEC = 0x8;
        /// Do not honour set-user-ID and set-group-ID bits when executing programs from this
        /// filesystem.
        const MS_NOSUID = 0x2;
        /// Mount filesystem read-only.
        const MS_RDONLY = 0x1;
        /// Used with [`MS_BIND`] to create a recursive bind mount.
        const MS_REC = 0x4_000;
        /// Only update access time if it's less than the last modification time.
        const MS_RELATIME = 0x200_000;
        /// Suppress certain warning messages in the kernel log.
        const MS_SILENT = 0x8_000;
        /// Always update the last access time, overriding [`MS_NOATIME`] and [`MS_RELATIME`].
        const MS_STRICTATIME = 0x1_000_000;
        /// Make writes on this filesystem synchronous.
        const MS_SYNCHRONOUS = 0x10;
        /// Do not follow symlinks when resolving paths.
        const MS_NOSYMFOLLOW = 0x100;
    }
}
impl Default for MountFlags {
    fn default() -> Self {
        Self::empty()
    }
}

bitflags::bitflags! {
    /// All the different flags which can be sent to the [`unmount`] function.
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct UmountFlags: i32 {
        /// Abort pending requests before attempting the unmount.
        const MNT_FORCE = 0x1;
        /// Perform a lazy unmount.
        const MNT_DETACH = 0x2;
        /// Mark the mount as expired.
        const MNT_EXPIRE = 0x4;
        /// Don't dereference the target path if it's a symbolic link.
        const UMOUNT_NOFOLLOW = 0x8;
    }
}
impl Default for UmountFlags {
    fn default() -> Self {
        Self::empty()
    }
}

/// Attaches the given filesystem path to the given location path.
///
/// Internally, this function uses the
/// [`mount`](https://man7.org/linux/man-pages/man2/mount.2.html) Linux syscall.
///
/// # Errors
///
/// This function propagates any [`Errno`]s from the underlying `mount` syscall.
pub fn mount<NA: Into<NixString>, NB: Into<NixString>>(
    source: NA,
    target: NB,
    filesystem_type: FilesystemType,
    mount_flags: MountFlags,
) -> Result<(), Errno> {
    let source_ns: NixString = source.into();
    let target_ns: NixString = target.into();
    let fs_ns: NixString = filesystem_type.into();

    // SAFETY: The arguments are of the correct number and type. NixString type guarantees
    // null-termination and valid UTF-8. The FilesystemType enum restricts the possible values
    // which can be passed for the filesystem type. The MountFlags struct restricts the possible
    // values which can be used for mount flags.
    unsafe {
        syscall_result!(
            SyscallNum::Mount,
            source_ns.as_ptr(),
            target_ns.as_ptr(),
            fs_ns.as_ptr(),
            mount_flags.bits(),
            ptr::null::<usize>()
        )?;
    }

    Ok(())
}

/// Removes the attachment of the topmost filesystem mounted at the given path.
///
/// Internally, this function uses the
/// [`umount2`](https://man7.org/linux/man-pages/man2/umount2.2.html) Linux syscall.
///
/// # Errors
///
/// This function propagates any [`Errno`]s from the underlying `umount2` syscall.
pub fn umount<NS: Into<NixString>>(target: NS, umount_flags: UmountFlags) -> Result<(), Errno> {
    let target_ns: NixString = target.into();

    // SAFETY: The arguments are of the correct number and type. NixString guarantees
    // null-termination and valid UTF-8. UmountFlags restricts the possible values which can be
    // used for umount2 flags.
    unsafe {
        syscall_result!(SyscallNum::Umount2, target_ns.as_ptr(), umount_flags.bits())?;
    }

    Ok(())
}
