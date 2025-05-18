//! Module for filesystem operations.

/// Process-unique identifier for a file or other input/output resource.
/// [Wikipedia](https://en.wikipedia.org/wiki/File_descriptor)
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileDescriptor(usize);
