//! Functionality related to memory.

use crate::{Errno, SyscallNum, syscall_result};

/// Changes the location of the program break. Increasing the program break grows the heap size and
/// allocates memory to the process; decreasing the break shrinks the heap size and deallocates
/// memory.
///
/// This function grows or shrinks the program's data space by `change` bytes and returns the
/// new program break on success.
///
/// # Errors
///
/// This function returns [`Errno::Enomem`] under the following conditions:
///
/// - The value is unreasonable.
/// - The system doesn't have enough memory.
/// - The process exceeds its maximum data size.
pub fn change_program_break(change: isize) -> Result<usize, Errno> {
    let old_break = brk(0)?;

    // Apply the change to the current program break, raising an error if the operation would go
    // out of bounds
    let new_break = match change {
        change if change.is_negative() => old_break
            .checked_sub(change.unsigned_abs())
            .ok_or(Errno::Enomem),
        #[allow(clippy::cast_sign_loss)]
        _ => old_break.checked_add(change as usize).ok_or(Errno::Enomem),
    }?;

    brk(new_break)
}

/// Wrapper around the [brk](https://www.man7.org/linux/man-pages/man2/brk.2.html) Linux syscall.
///
/// Returns the new program break on success.
fn brk(brk_addr: usize) -> Result<usize, Errno> {
    // SAFETY: The `brk` syscall handles bad address values internally. The arguments are correct.
    unsafe { syscall_result!(SyscallNum::Brk, brk_addr) }
}
