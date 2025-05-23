//! Module for filesystem operations.

mod dirs;
mod file;
mod mount;
mod open_flags;
mod open_options;
mod permissions;
mod types;

// RE-EXPORTS
pub use dirs::{change_dir, get_cwd};
pub use file::File;
pub use mount::{FilesystemType, MountFlags, UmountFlags, mount, umount};
pub use open_flags::OpenFlags;
pub use open_options::OpenOptions;
pub use permissions::FilePermissions;
pub use types::{FileDescriptor, FileStat, FileStatRaw, FileType, LseekWhence};

#[cfg(test)]
mod tests;
