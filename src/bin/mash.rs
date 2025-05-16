//! `mash` = **Ma**x's **Sh**ell. Tlenix's shell program! Provides a command-line user interface
//! for Tlenix.

#![warn(
    missing_docs,
    missing_debug_implementations,
    rust_2018_idioms,
    clippy::all,
    clippy::pedantic
)]
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![cfg_attr(test, test_runner(tlenix_core::custom_test_runner))]

use core::panic::PanicInfo;

const MASH_PANIC_TITLE: &str = "mash";

/// Entry point.
///
/// # Panics
///
/// This function panics if the system fails to power off properly.
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    // Align stack pointer
    //
    // SAFETY: Valid ASM instruction with valid, statically-chosen arguments.
    unsafe {
        core::arch::asm!("and rsp, -16", options(nostack));
    }

    // TODO use a better loop
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    tlenix_core::eprintln!("{} {}", MASH_PANIC_TITLE, info);
    // TODO exit with failure
    loop {}
}
