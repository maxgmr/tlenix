//! Module for filesystem operations.

mod file;
mod open_flags;
mod open_options;
mod permissions;
mod types;

// RE-EXPORTS
pub use file::File;
pub use open_flags::OpenFlags;
pub use open_options::OpenOptions;
pub use permissions::FilePermissions;
pub use types::{FileDescriptor, FileStat, FileStatRaw, FileType, LseekWhence};

#[cfg(test)]
mod tests;
