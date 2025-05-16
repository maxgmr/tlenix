//! Functionality related to process management.

use crate::{ExitStatus, SyscallNum, syscall};

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
