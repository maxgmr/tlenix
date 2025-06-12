//! The [`FileDescriptor`] type.

/// Process-unique identifier for a file or other input/output resource.
/// [Wikipedia](https://en.wikipedia.org/wiki/File_descriptor)
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileDescriptor(usize);
impl FileDescriptor {
    /// Defines a [`FileDescriptor`] with the given `usize`.
    #[doc(hidden)]
    pub(crate) const fn define(value: usize) -> Self {
        Self(value)
    }
}
impl From<usize> for FileDescriptor {
    fn from(value: usize) -> Self {
        Self::define(value)
    }
}
impl From<FileDescriptor> for usize {
    fn from(value: FileDescriptor) -> Self {
        value.0
    }
}
