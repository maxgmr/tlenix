//! Functionality related to the computer system itself.

use crate::{Errno, SyscallNum, syscall_result};

const LINUX_REBOOT_MAGIC1: usize = 0xfee1_dead;
const LINUX_REBOOT_MAGIC2C: usize = 0x2011_2000;

/// The different operations which can be performed by the [reboot](https://man7.org/linux/man-pages/man2/reboot.2.html) Linux syscall.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
#[allow(dead_code)]
enum RebootCmd {
    CadOff = 0,
    CadOn = 0x89ab_cdef,
    Halt = 0xcdef_0123,
    Kexec = 0x4558_4543,
    PowerOff = 0x4321_fedc,
    Restart = 0x0123_4567,
    Restart2 = 0xa1b2_c3d4,
    SwSuspend = 0xd000_fce1,
}

/// Attempts to reboot the computer.
///
/// # Errors
///
/// This function propagates the error from the underlying [reboot](https://man7.org/linux/man-pages/man2/reboot.2.html)
/// Linux syscall if the system fails to reboot.
///
/// # Panics
///
/// This function panics if the underlying system call somehow returns a success but fails to
/// reboot the system.
pub fn reboot() -> Result<!, Errno> {
    reboot_syscall(RebootCmd::Restart)
}

/// Attempts to power off the computer.
///
/// # Errors
///
/// This function propagates the error from the underlying [reboot](https://man7.org/linux/man-pages/man2/reboot.2.html)
/// Linux syscall if the system fails to power off.
///
/// # Panics
///
/// This function panics if the underlying system call somehow returns a success but fails to power
/// off the system.
pub fn power_off() -> Result<!, Errno> {
    reboot_syscall(RebootCmd::PowerOff)
}

/// Wrapper for the [reboot](https://man7.org/linux/man-pages/man2/reboot.2.html) syscall.
///
/// Performs the given [`RebootCmd`].
///
/// # Errors
///
/// This function errors if:
///
/// - Problem getting user-space data under [`RebootCmd::Restart2`].
/// - Bad magic numbers or `operation`.
/// - The calling process has insufficient privilege to call `reboot`.
///
/// # Panics
///
/// This function panics if reboot returns a success (this function is only intended to be used
/// with `operation` values that stop or restart the system).
fn reboot_syscall(operation: RebootCmd) -> Result<!, Errno> {
    // SAFETY: Arguments are correct, and the values passable to the `op` argument are restricted
    // to correct ones by the `RebootCmd` enum.
    unsafe {
        Err(syscall_result!(
            SyscallNum::Reboot,
            LINUX_REBOOT_MAGIC1,
            LINUX_REBOOT_MAGIC2C,
            operation as usize,
            "".as_ptr() as usize
        )
        .expect_err("reboot syscall somehow returned success"))
    }
}
