//! Library crate for the [tlenix](https://github.com/maxgmr/tlenix) `x86_64` operating system.
#![warn(
    missing_docs,
    missing_debug_implementations,
    rust_2018_idioms,
    clippy::all,
    clippy::pedantic,
    clippy::unwrap_used
)]
#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(test_framework::custom_test_runner)]
#![reexport_test_harness_main = "test_main"]

// Make sure the compiler includes `alloc`
#[allow(unused_extern_crates)]
extern crate alloc;

mod allocator;
mod print;
mod syscall;
mod test_framework;

// RE-EXPORTS
pub use print::{__print_err, __print_str};
pub use syscall::{Errno, SyscallNum};
pub use test_framework::custom_test_runner;

/// C standard success exit code.
pub const EXIT_SUCCESS: usize = 0;
/// C standard failure exit code.
pub const EXIT_FAILURE: usize = 1;

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

/// Entry point for library tests.
///
/// # Panics
///
/// This function panics if the sleep loop returns an error.
#[cfg(test)]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    align_stack_pointer!();
    test_main();

    // TODO process exit successfully
    #[allow(clippy::empty_loop)]
    loop {}
}

/// Panic handler for library tests.
#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    test_framework::test_panic_handler(info)
}
