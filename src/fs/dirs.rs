//! Functionality related to directories.

use alloc::{string::String, vec::Vec};

use crate::{
    Errno, NULL_BYTE, SyscallNum, fs::FilePermissions, nix_str::NixString, syscall_result,
};

const INITIAL_CWD_BUF_SIZE: usize = 1 << 8;

/// Changes the current directory of the process to the given `path`.
///
/// Wrapper around the [`chdir`](https://man7.org/linux/man-pages/man2/chdir.2.html) Linux syscall.
///
/// # Errors
///
/// This function propagates any [`Errno`]s returned by the underlying call to `chdir`.
pub fn change_dir<NS: Into<NixString>>(path: NS) -> Result<(), Errno> {
    let nstring = path.into();

    // SAFETY: The arguments are valid. The pointer to nstring is dropped right away.
    unsafe {
        syscall_result!(SyscallNum::Chdir, nstring.as_ptr())?;
    }
    Ok(())
}

/// Gets the current working directory of the process.
///
/// Wrapper around the [`getcwd`](https://man7.org/linux/man-pages/man2/getcwd.2.html) Linux
/// syscall.
///
/// # Errors
///
/// This function propagates any [`Errno`]s returned by the underlying call to `getcwd`.
///
/// Additionally, it returns [`Errno::Eilseq`] if the path is not valid UTF-8.
pub fn get_cwd() -> Result<String, Errno> {
    let mut buffer: Vec<u8> = Vec::with_capacity(INITIAL_CWD_BUF_SIZE);

    // Keep trying to fit the cwd string into the buffer, reallocating if it's too small.
    loop {
        // Ensure the buffer size matches its capacity
        buffer.resize(buffer.capacity(), 0);
        // SAFETY: The arguments are valid. The buffer capacity is programmatically determined and
        // guaranteed to match the buffer itself. Finally, the pointer to the buffer isn't used
        // after the buffer is reallocated.
        match unsafe { syscall_result!(SyscallNum::Getcwd, buffer.as_mut_ptr(), buffer.len()) } {
            // Got it! return the buffer as a string.
            Ok(_) => break,
            // Too small. Double the size and try again.
            Err(Errno::Erange) => {
                buffer.reserve(buffer.capacity());
            }
            // Other error. Return it.
            Err(e) => return Err(e),
        }
    }

    // Trim null bytes
    let len = buffer
        .iter()
        .position(|&byte| byte == NULL_BYTE)
        .unwrap_or(buffer.len());
    buffer.truncate(len);

    String::from_utf8(buffer).map_err(|_| Errno::Eilseq)
}

/// Attempts to create a new directory with the given path.
///
/// Additionally, the mode of the directory is specified with the given [`FilePermissions`].
///
/// Internally uses the [`mkdir`](https://man7.org/linux/man-pages/man2/mkdir.2.html) Linux
/// syscall.
///
/// # Errors
///
/// This function propagates any [`Errno`]s returned by the call to `mkdir`.
pub fn mkdir<NS: Into<NixString>>(path: NS, mode: FilePermissions) -> Result<(), Errno> {
    let ns_path: NixString = path.into();
    // SAFETY: The mode is restricted by the FilePermissions type. The NixString type guarantees
    // null-termination and UTF-8 validity of the given string.
    unsafe {
        syscall_result!(SyscallNum::Mkdir, ns_path.as_ptr(), mode.bits())?;
    }
    Ok(())
}

/// Attempts to delete the directory at the given path. This directory must be empty.
/// Internally uses the [`rmdir`](https://man7.org/linux/man-pages/man2/rmdir.2.html) Linux
/// syscall.
///
/// # Errors
///
/// This function propagates any [`Errno`]s returned by the call to `rmdir`.
pub fn rmdir<NS: Into<NixString>>(path: NS) -> Result<(), Errno> {
    let ns_path: NixString = path.into();
    // SAFETY: The NixString type guarantees null-termination and UTF-8 validity of the given
    // string.
    unsafe {
        syscall_result!(SyscallNum::Rmdir, ns_path.as_ptr())?;
    }
    Ok(())
}
