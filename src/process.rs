//! Functionality related to process management.

use alloc::vec::Vec;
use core::ptr;

use crate::{Errno, SyscallNum, data::NullTermString, syscall, syscall_result};

const WUNTRACED: usize = 0x2;

/// Create a child process running the executable at the given [`NullTermStr`]. The parent process
/// which calls this function waits until the child process is exited or killed.
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
pub fn execute_process(argv: &[NullTermString]) -> Result<(), Errno> {
    // Return ENOENT if no path is given
    if argv.is_empty() {
        return Err(Errno::Enoent);
    }

    match fork()? {
        0 => {
            // Child process; start the given program!

            // Get pointers to all the args
            let mut argv_pointers: Vec<*const u8> =
                argv.iter().map(NullTermString::as_ptr).collect();
            // Null-terminate argv
            argv_pointers.push(ptr::null());
            // Get pointer to start of pointer arr
            let argv_ptr = argv_pointers.as_ptr();

            // TODO: handle envp
            let envp: [*const u8; 1] = [ptr::null()];
            let envp_ptr = envp.as_ptr();

            // SAFETY: On success, `execve` does not return, so the pointers only need to be valid at
            // the moment of the syscall.
            unsafe {
                syscall_result!(
                    SyscallNum::Execve,
                    argv_pointers[0] as usize,
                    argv_ptr as usize,
                    envp_ptr as usize
                )
                .unwrap();
            }
            unreachable!("execve should have panicked on fail");
        }
        child_pid => {
            // Parent process; wait for child to finish!

            let mut status: usize = 0;
            unsafe {
                syscall_result!(
                    SyscallNum::Wait4,
                    child_pid,
                    &raw mut status as usize,
                    WUNTRACED,
                    0
                )?;
            }

            // Done waiting; continue
            Ok(())
        }
    }
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
