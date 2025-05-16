//! The `init` program for `tlenix`. Expected location is at `/sbin/init` so the Linux kernel can
//! call it after boot.

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

use tlenix_core::{align_stack_pointer, sleep_loop_forever};

#[cfg(not(test))]
const WELCOME_MSG: &str = concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION"));
const TLENIX_PANIC_TITLE: &str = "tlenix";

/// Entry point.
///
/// # Panics
///
/// This function panics if the system fails to power off properly. This is intended behaviour for
/// a Linux-based init program.
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    align_stack_pointer!();

    // Don't do anything if we're running tests
    #[cfg(test)]
    tlenix_core::process::exit(tlenix_core::ExitStatus::ExitSuccess);

    #[cfg(not(test))]
    {
        welcome_msg();

        sleep_loop_forever();
    }
}

#[cfg(not(test))]
fn welcome_msg() {
    tlenix_core::println!("{}", WELCOME_MSG);
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    tlenix_core::eprintln!("{} {}", TLENIX_PANIC_TITLE, info);
    sleep_loop_forever();
}
