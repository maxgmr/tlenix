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
#![feature(never_type)]
#![feature(custom_test_frameworks)]
#![feature(concat_bytes)]
#![test_runner(test_framework::custom_test_runner)]
#![reexport_test_harness_main = "test_main"]

pub mod consts;
pub mod data;
pub mod fs;
pub mod io;
pub mod process;
pub mod syscalls;
pub mod system;
pub mod thread;

mod test_framework;

// Re-exports

pub use syscalls::{Errno, SyscallNum};

pub use test_framework::custom_test_runner;

/// Entry point for library tests.
///
/// # Panics
///
/// This function panics if the sleep loop returns an error.
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

    sleep_loop().unwrap()
}

/// Endlessly loop, sleeping the thread.
///
/// # Errors
///
/// This function returns an error if [`thread::sleep`] returns an error.
pub fn sleep_loop() -> Result<!, Errno> {
    let sleep_duration = core::time::Duration::from_nanos(consts::PIT_IRQ_PERIOD);
    loop {
        thread::sleep(&sleep_duration)?;
    }
}

/// Endlessly loop, sleeping the thread.
///
/// If [`thread::sleep`] returns an error for whatever reason, an empty loop is used as a fallback.
pub fn sleep_loop_forever() -> ! {
    let _ = sleep_loop();
    // Fallback loop if `sleep_loop` breaks
    #[allow(clippy::empty_loop)]
    loop {}
}

/// Panic handler for library tests.
#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    test_framework::test_panic_handler(info)
}
