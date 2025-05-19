//! Module for filesystem operations.

mod file;
mod open_flags;
mod open_options;
mod permissions;

// RE-EXPORTS
pub use file::{File, FileDescriptor};
pub use open_flags::OpenFlags;
pub use open_options::OpenOptions;
pub use permissions::FilePermissions;

#[cfg(test)]
mod tests;
