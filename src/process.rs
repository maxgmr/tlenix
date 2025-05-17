//! Functionality related to process management.

use crate::{Errno, ExitStatus, SyscallNum, syscall, syscall_result};

/// Create a child process running the executable at the given filepath. The parent process which
/// calls this function waits until the child process is exited or killed.
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
pub fn execute_process() {
    todo!()
}

/// Cause normal process termination. Wrapper around the
/// [exit](https://www.man7.org/linux/man-pages/man3/exit.3.html) Linux syscall.
///
/// Returns the least significant byte of the given `status` to the parent process.
pub fn exit(exit_status: ExitStatus) -> ! {
    // SAFETY: The only user-defined argument, `status`, is already the right type.
    unsafe {
        syscall!(SyscallNum::Exit, exit_status);
    }
    unreachable!("failed to exit somehow")
}

/// Creates a child process. Wrapper around the [fork](https://www.man7.org/linux/man-pages/man2/fork.2.html) Linux syscall.
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
