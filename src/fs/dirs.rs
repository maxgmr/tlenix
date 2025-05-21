//! Functionality related to directories.

use alloc::{string::String, vec::Vec};

use crate::{Errno, NULL_BYTE, SyscallNum, nix_str::NixString, syscall_result};

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
