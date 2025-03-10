//! Functionality related to memory.

use core::ptr;

use talc::{ClaimOnOom, Span, Talc, Talck};

use crate::{Errno, SyscallNum, syscall_result};

// Talc global memory allocator
static mut ARENA: [u8; 16_384] = [0; 16_384];

#[global_allocator]
static ALLOCATOR: Talck<spin::Mutex<()>, ClaimOnOom> =
    Talc::new(unsafe { ClaimOnOom::new(Span::from_array(ptr::addr_of!(ARENA).cast_mut())) }).lock();

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

#[cfg(test)]
mod tests {
    use alloc::{string::String, vec::Vec};

    use super::*;

    #[test_case]
    #[allow(clippy::cast_sign_loss)]
    fn alloc_and_dealloc() {
        // 4 KiB
        let increase = 4096;
        let decrease = -4096;

        let initial_break = change_program_break(0).unwrap();

        let new_break = change_program_break(increase).unwrap();
        assert_eq!(new_break, initial_break + (increase as usize));

        let new_break = change_program_break(decrease).unwrap();
        assert_eq!(initial_break, new_break);
    }

    #[test_case]
    #[allow(clippy::cast_possible_wrap)]
    fn oob() {
        let current_break = change_program_break(0).unwrap();
        let takeaway = -((current_break as isize) + 1);
        assert_eq!(change_program_break(takeaway), Err(Errno::Enomem));
    }

    #[test_case]
    fn my_box() {
        let _ = alloc::boxed::Box::new(42);
    }

    #[test_case]
    fn my_vec() {
        let mut my_vec: Vec<i32> = Vec::new();

        assert!(my_vec.is_empty());
        assert_eq!(my_vec.len(), 0);

        my_vec.push(1293);
    }

    #[test_case]
    fn my_string() {
        use alloc::string::ToString;

        let mut my_string = String::new();
        my_string.push('H');
        my_string.push('e');
        my_string.push('l');
        my_string.push_str("lo, world!");

        let mut my_string2 = "It's nice to".to_string();
        let my_string3 = " meet you!".to_string();
        my_string2.push_str(&my_string3);
    }
}
