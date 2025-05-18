//! Module for filesystem operations.

mod mode_t;
mod open_flags;
mod open_options;

// RE-EXPORTS
pub use mode_t::ModeT;
pub use open_flags::OpenFlags;
pub use open_options::OpenOptions;

/// Process-unique identifier for a file or other input/output resource.
/// [Wikipedia](https://en.wikipedia.org/wiki/File_descriptor)
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileDescriptor(usize);

#[cfg(test)]
mod tests;
