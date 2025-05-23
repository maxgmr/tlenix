//! Functionality related to process management.

use alloc::vec::Vec;
use core::ptr;

use crate::{Errno, ExitStatus, NixBytes, SyscallNum, syscall, syscall_result, vec_into_nix_bytes};

const WUNTRACED: usize = 2;

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
pub fn execute_process<T: Into<NixBytes> + Clone, U: Into<NixBytes> + Clone>(
    argv: Vec<T>,
    envp: Vec<U>,
) -> Result<(), Errno> {
    // Return ENOENT if no path is given
    if argv.is_empty() {
        return Err(Errno::Enoent);
    }

    // ARGV
    // Convert to syscall-compatible strings
    let argv_nix_strings: Vec<NixBytes> = vec_into_nix_bytes(argv);
    // Get an array of pointers to those strings
    let mut argv_pointers: Vec<*const u8> = argv_nix_strings.iter().map(NixBytes::as_ptr).collect();
    // Null-terminate the array
    argv_pointers.push(ptr::null());
    // Get pointer to start of argv array
    let argv_pointer = argv_pointers.as_ptr();

    // ENVP
    // Convert to syscall-compatible strings
    let envp_nix_strings: Vec<NixBytes> = vec_into_nix_bytes(envp);
    // Get an array of pointers to those strings
    let mut envp_pointers: Vec<*const u8> = envp_nix_strings.iter().map(NixBytes::as_ptr).collect();
    // Null-terminate the array
    envp_pointers.push(ptr::null());
    // Get pointer to start of envp array
    let envp_pointer = envp_pointers.as_ptr();

    match fork()? {
        0 => {
            // Child process; start the given program

            // SAFETY: On success, `execve` does not return, so the pointers only need to be valid
            // at the moment of the syscall (which they are). Furthermore, the child process
            // immediately exits if `execve` fails, avoiding UB there.
            unsafe {
                if syscall_result!(
                    SyscallNum::Execve,
                    argv_pointers[0],
                    argv_pointer,
                    envp_pointer
                )
                .is_err()
                {
                    exit(ExitStatus::ExitFailure);
                }
            }
            unreachable!("execve doesn't return on success");
        }
        child_pid => {
            // Parent process; wait for child to finish
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
/// Returns the least significant byte of the given `exit_status` to the parent process.
pub fn exit(exit_status: ExitStatus) -> ! {
    // SAFETY: The only user-defined argument, `exit_status`, is already the right type.
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
