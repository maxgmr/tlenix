//! Library crate for the [tlenix](https://github.com/maxgmr/tlenix) `x86_64` operating system.
#![warn(
    missing_docs,
    missing_debug_implementations,
    rust_2018_idioms,
    clippy::all,
    clippy::pedantic
)]
#![no_std]

/// Aligns the stack pointer. Intended for use right at the beginning of execution.
///
/// SAFETY: Valid ASM instruction with valid, statically-chosen arguments.
#[macro_export]
macro_rules! align_stack_pointer {
    // This can't be called as a function; it must be directly invoked right at the start.
    () => {
        unsafe {
            core::arch::asm!("and rsp, -16", options(nostack));
        }
    };
}
