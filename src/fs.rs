//! Module for filesystem operations.

mod dirs;
mod file;
mod mount;
mod open_flags;
mod open_options;
mod permissions;
mod types;

// RE-EXPORTS
pub use dirs::{change_dir, chroot, get_cwd, mkdir, rmdir};
pub use file::{File, rename, rm};
pub use mount::{FilesystemType, MountFlags, UmountFlags, mount, pivot_root, umount};
pub use open_flags::OpenFlags;
pub use open_options::OpenOptions;
pub use permissions::FilePermissions;
pub use types::{
    DirEnt, FileAttributes, FileDescriptor, FileStats, FileStatsMask, FileType, LseekWhence,
};
pub(crate) use types::{FileStatsRaw, statx_get_all};

#[cfg(test)]
mod tests;
