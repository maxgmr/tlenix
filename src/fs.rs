//! Module for filesystem operations.

use crate::{Errno, SyscallNum, nix_str::NixString, syscall_result};

mod mode_t;
mod open_flags;

// RE-EXPORTS
pub use mode_t::ModeT;
pub use open_flags::OpenFlags;

/// Process-unique identifier for a file or other input/output resource.
/// [Wikipedia](https://en.wikipedia.org/wiki/File_descriptor)
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileDescriptor(usize);

/// Opens an existing file at the given path, returning its [`FileDescriptor`].
///
/// Uses the [open](https://www.man7.org/linux/man-pages/man2/open.2.html) Linux syscall.
///
/// To ensure safety, this function cannot be used to create a file or a tempfile. This is because
/// this function does not provide a `mode` syscall argument.
///
/// # Errors
///
/// This function returns an [`Errno`] when:
///
/// 1. `O_CREAT` or `O_TMPFILE` flags are set. In this case, [`Errno::Eperm`] is returned.
/// 2. `O_RDONLY` and `O_TRUNC` flags are set. In this case, [`Errno::Eperm`] is returned.
/// 3. The underlying [open](https://www.man7.org/linux/man-pages/man2/open.2.html) syscall returns
///    an [`Errno`].
pub fn open<S: Into<NixString>>(path: S, flags: &OpenFlags) -> Result<FileDescriptor, Errno> {
    if flags.intersects(OpenFlags::O_CREAT)
        || flags.contains(OpenFlags::O_RDONLY | OpenFlags::O_TRUNC)
    {
        return Err(Errno::Eperm);
    }

    let path: NixString = path.into();
    let raw_fd = unsafe { syscall_result!(SyscallNum::Open, path.as_ptr(), flags.bits())? };
    Ok(FileDescriptor(raw_fd))
}

#[cfg(test)]
mod tests;
