//! Library crate for the [tlenix](https://github.com/maxgmr/tlenix) `x86_64` operating system.
#![warn(
    missing_docs,
    missing_debug_implementations,
    rust_2018_idioms,
    clippy::all,
    clippy::pedantic
)]
#![no_std]
#![cfg_attr(test, no_main)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(custom_test_frameworks)]
#![test_runner(test_framework::custom_test_runner)]
#![reexport_test_harness_main = "test_main"]

#[cfg(test)]
use core::panic::PanicInfo;

pub mod consts;
pub mod data;
pub mod fs;
pub mod io;
pub mod syscalls;
pub mod system;

mod test_framework;

// Re-exports

pub use syscalls::{Errno, SyscallNum};

pub use test_framework::custom_test_runner;

/// Entry point for library tests.
///
/// # Panics
///
/// Panics if the system fails to power off.
#[cfg(test)]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    // Align stack pointer
    //
    // SAFETY: Valid ASM instruction with valid, statically-chosen arguments.
    unsafe {
        core::arch::asm!("and rsp, -16", options(nostack));
    }
    test_main();

    // TODO replace with a better loop
    #[allow(clippy::empty_loop)]
    loop {}
}

/// Panic handler for library tests.
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    test_framework::test_panic_handler(info)
}
