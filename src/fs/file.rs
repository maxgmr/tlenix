//! This module is responsible for the [`File`] type and all associated file operations.

use crate::fs::OpenOptions;

/// Process-unique identifier for a file or other input/output resource.
/// [Wikipedia](https://en.wikipedia.org/wiki/File_descriptor)
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileDescriptor(usize);
impl From<usize> for FileDescriptor {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

/// An object providing access to an open file on the filesystem.
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct File {
    file_descriptor: FileDescriptor,
    open_options: OpenOptions,
}
impl File {
    /// Creates a [`File`] at the given [`FileDescriptor`] with the given open options. Not
    /// intended to be used directly.
    #[doc(hidden)]
    #[must_use]
    pub(crate) fn __new(file_descriptor: FileDescriptor, open_options: &OpenOptions) -> Self {
        Self {
            file_descriptor,
            open_options: open_options.clone(),
        }
    }
}
