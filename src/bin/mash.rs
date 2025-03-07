//! `mash` = **Ma**x's **Sh**ell. Tlenix's shell program! Provides a command-line user interface
//! for Tlenix.

#![no_std]
#![no_main]
#![warn(
    missing_docs,
    missing_debug_implementations,
    rust_2018_idioms,
    clippy::all,
    clippy::pedantic
)]
#![feature(custom_test_frameworks)]
#![cfg_attr(test, test_runner(tlenix_core::custom_test_runner))]

use core::panic::PanicInfo;

use tlenix_core::{println, sleep_loop, sleep_loop_forever};

const WELCOME_MSG: &str = "Welcome to MASH!";
const TLENIX_PANIC_TITLE: &str = "tlenix";

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
    welcome_msg();

    // TODO

    sleep_loop().unwrap()
}

fn welcome_msg() {
    println!("{WELCOME_MSG}");
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    tlenix_core::eprintln!("{} {}", TLENIX_PANIC_TITLE, info);
    sleep_loop_forever()
}
