//! Functionality related to process management.

use alloc::vec::Vec;
use core::ptr;

use crate::{Errno, NixString, SyscallNum, ipc::SigInfoRaw, syscall, syscall_result};

mod types;

pub use types::{ExitStatus, WaitIdType, WaitInfo, WaitOptions};

/// Arguments formatted for `execve`.
struct ExecArgs {
    /// The arguments themselves, guaranteed to be null-terminated, valid UTF-8 bytes.
    strs: Vec<NixString>,
    /// Pointers to each of the arguments. Null-terminated.
    ptrs: Vec<*const u8>,
}
impl ExecArgs {
    /// Creates a new [`ExecArgs`] from the given slice.
    fn from_slice<NS: Into<NixString> + Clone>(slice: &[NS]) -> Self {
        let mut strs = Vec::with_capacity(slice.len());
        let mut ptrs = Vec::with_capacity(slice.len());

        for elem in slice {
            let str: NixString = elem.clone().into();
            ptrs.push(str.as_ptr());
            strs.push(str);
        }

        // Make sure it's null-terminated!
        ptrs.push(ptr::null());

        Self { strs, ptrs }
    }

    /// Returns a pointer to the start of this [`ExecArgs`]' pointer list.
    fn as_ptr(&self) -> *const *const u8 {
        self.ptrs.as_ptr()
    }

    /// Returns a pointer to the `n`th [`NixString`] of this [`ExecArgs`].
    fn ptr_to_string(&self, n: usize) -> Option<*const u8> {
        if n >= self.strs.len() {
            return None;
        }

        // SAFETY: We just checked that `n` was in bounds.
        Some(self.strs[n].as_ptr())
    }
}

/// Executes the program referred to by the given file name, causing the current process to be
/// replaced by the new one.
///
/// The name of the program is the first element of `argv`, while the other elements of `argv` are
/// the arguments sent to the program.
///
/// `envp` is a list of environment variables, conventionally of the form `key=value`.
///
/// This function does not return on success.
///
/// Internally, this function uses the
/// [`execve`](https://man7.org/linux/man-pages/man2/execve.2.html) Linux syscall.
///
/// # Errors
///
/// This function returns [`Errno::Enoent`] if the `argp` slice is empty.
///
/// This function propagates any [`Errno`]s returned by the underlying call to [`execve`].
// Function won't panic. See below.
#[allow(clippy::missing_panics_doc)]
pub fn execve<NA: Into<NixString> + Clone, NB: Into<NixString> + Clone>(
    argv: &[NA],
    envp: &[NB],
) -> Result<!, Errno> {
    if argv.is_empty() {
        return Err(Errno::Enoent);
    }
    let argv_exec_args = ExecArgs::from_slice(argv);
    let envp_exec_args = ExecArgs::from_slice(envp);
    // OK to unwrap here- we already made sure argv wasn't empty.
    #[allow(clippy::unwrap_used)]
    let filename = argv_exec_args.ptr_to_string(0).unwrap();

    // SAFETY: On success, `execve` does not return, so the pointers only need to be valid
    // at the moment of the syscall (which they are). Potential UB on failure is caught gracefully.
    // The `NixBytes` type guarantees that all strings are null-terminated. Both pointer arrays are
    // null-terminated in the above code.
    unsafe {
        syscall_result!(
            SyscallNum::Execve,
            filename,
            argv_exec_args.ptrs.as_ptr(),
            envp_exec_args.ptrs.as_ptr()
        )?;
    }
    unreachable!("execve doesn't return on success");
}

/// Creates a child process running the executable at the given file name. The parent process which
/// calls this function waits until the child process is exited or killed. Finally, the
/// [`ExitStatus`] of the child process is returned.
///
/// The name of the program is the first element of `argv`, while the other elements of `argv` are
/// the arguments sent to the program.
///
/// `envp` is a list of environment variables, conventionally of the form `key=value`.
///
/// # Errors
///
/// This function returns [`Errno::Enoent`] if `argv` is empty.
///
/// This function propagates any [`Errno`]s returned by the underlying calls to
/// [`fork`](https://www.man7.org/linux/man-pages/man2/fork.2.html) and
/// [`execve`](https://man7.org/linux/man-pages/man2/execve.2.html).
// Function won't panic. See below.
#[allow(clippy::missing_panics_doc)]
pub fn execute_process<NA: Into<NixString> + Clone, NB: Into<NixString> + Clone>(
    argv: &[NA],
    envp: &[NB],
) -> Result<ExitStatus, Errno> {
    if argv.is_empty() {
        return Err(Errno::Enoent);
    }
    let argv_exec_args = ExecArgs::from_slice(argv);
    let envp_exec_args = ExecArgs::from_slice(envp);
    // OK to unwrap here- we already made sure argv wasn't empty.
    #[allow(clippy::unwrap_used)]
    let filename = argv_exec_args.ptr_to_string(0).unwrap();

    match fork()? {
        0 => {
            // Child process; start the given program

            // SAFETY: On success, `execve` does not return, so the pointers only need to be valid
            // at the moment of the syscall (which they are). Furthermore, the child process
            // immediately exits if `execve` fails, avoiding UB there.
            if let Err(errno) = unsafe {
                syscall_result!(
                    SyscallNum::Execve,
                    filename,
                    argv_exec_args.as_ptr(),
                    envp_exec_args.as_ptr()
                )
            } {
                exit(ExitStatus::ExitFailure(errno as i32));
            }
            unreachable!("execve doesn't return on success");
        }
        child_pid => {
            // Parent process; wait for child to finish
            let wait_info = wait(child_pid, WaitIdType::Pid, WaitOptions::WEXITED)?;
            wait_info.try_into()
        }
    }
}

/// Waits for the given process (or group of processes) to change state.
///
/// Internally uses the [`waitid`](https://man7.org/linux/man-pages/man2/waitid.2.html) Linux
/// system call.
///
/// # Errors
///
/// This function propagates any [`Errno`]s returned by the underlying call to `waitid`.
pub fn wait(id: usize, id_type: WaitIdType, wait_options: WaitOptions) -> Result<WaitInfo, Errno> {
    let mut sig_info_raw = SigInfoRaw::default();

    // SAFETY: WaitIdType restricts the given values to valid ones. SigInfoRaw matches the layout
    // of `siginfo_t`. WaitOptions restricts the given values to valid ones. A null pointer is given for the last argument.
    unsafe {
        syscall_result!(
            SyscallNum::Waitid,
            id_type as u32,
            id,
            &raw mut sig_info_raw,
            wait_options.bits(),
            core::ptr::null::<u8>()
        )?;
    }

    WaitInfo::try_from(sig_info_raw)
}

/// Causes normal process termination. Wrapper around the
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
