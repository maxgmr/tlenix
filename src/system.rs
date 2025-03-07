//! Functionality related to the computer system itself.

use super::{SyscallNum, consts::SYSCALL_FAIL, syscall};

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

/// Attempt to power off the computer.
///
/// # Panics
///
/// This function panics if it is unable to shut down the system.
pub fn expect_power_off() {
    assert!(power_off_syscall() != SYSCALL_FAIL, "Failed to power off!");
}

/// Wrapper for the [reboot](https://man7.org/linux/man-pages/man2/reboot.2.html) syscall with the
/// `op` argument set to `LINUX_REBOOT_CMD_POWER_OFF`.
///
/// Stop the system and remove power from the system (if possible).
#[must_use]
fn power_off_syscall() -> i32 {
    // SAFETY
    unsafe {
        syscall!(
            SyscallNum::Reboot,
            LINUX_REBOOT_MAGIC1,
            LINUX_REBOOT_MAGIC2C,
            RebootCmd::PowerOff as usize,
            "".as_ptr() as usize
        );
    }
    // Shouldn't return!
    SYSCALL_FAIL
}
