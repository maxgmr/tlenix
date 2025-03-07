//! Functionality related to process management.

use core::ptr;

use crate::{Errno, SyscallNum, data::NullTermStr, syscall, syscall_result};

/// Create a child process running the executable at the given [`NullTermStr`].
///
/// # Errors
///
/// This function propagates any [`Errno`] returned by the
/// [fork](https://www.man7.org/linux/man-pages/man2/fork.2.html) Linux syscall or the
/// [execve](https://man7.org/linux/man-pages/man2/execve.2.html) Linux syscall.
///
/// # Panics
///
/// This function panics if the child process attempts to call `execve` and it fails.
pub fn execute_process<const N: usize>(path: &NullTermStr<N>) -> Result<(), Errno> {
    let pid = fork()?;

    if pid == 0 {
        // Child process, start the given program!

        // TODO: handle passing actual args
        let argv: [*const u8; 2] = [path.as_ptr(), ptr::null()];
        let envp: [*const u8; 1] = [ptr::null()];

        let argv_ptr = argv.as_ptr();
        let envp_ptr = envp.as_ptr();

        // SAFETY: On success, `execve` does not return, so the pointers only need to be valid at
        // the moment of the syscall.
        unsafe {
            syscall_result!(
                SyscallNum::Execve,
                path.as_ptr() as usize,
                argv_ptr as usize,
                envp_ptr as usize
            )
            .unwrap();
        }
        unreachable!("execve should have panicked on fail");
    }

    Ok(())
}

/// Cause normal process termination. Wrapper around the
/// [exit](https://www.man7.org/linux/man-pages/man3/exit.3.html) Linux syscall.
///
/// Returns the least significant byte of the given `status` to the parent process.
pub fn exit(status: usize) -> ! {
    // SAFETY: The only user-defined argument, `status`, is already the right type.
    unsafe {
        syscall!(SyscallNum::Exit, status);
    }
    unreachable!("failed to exit somehow")
}

/// Create a child process. Wrapper around the [fork](https://www.man7.org/linux/man-pages/man2/fork.2.html) Linux syscall.
///
/// On success, the PID of the child process is returned in the parent, and 0 is returned in the
/// child.
///
/// # Errors
///
/// This function returns an [`Errno`] if the underlying syscall fails.
fn fork() -> Result<usize, Errno> {
    // SAFETY: This syscall has no arguments, and errors are handled gracefully.
    unsafe { syscall_result!(SyscallNum::Fork) }
}
