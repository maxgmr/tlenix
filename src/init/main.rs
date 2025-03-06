//! The `init` program for `tlenix`. Expected location is at `/sbin/init` so the Linux kernel can
//! call it after boot.

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

use tlenix_core::println;

const WELCOME_MSG: &str = concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION"));
const TLENIX_PANIC_TITLE: &str = "tlenix";

/// Entry point.
///
/// # Panics
///
/// This function panics if the system fails to power off properly.
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    welcome_msg();

    for _ in 0..100_000_000 {
        core::hint::spin_loop();
    }

    tlenix_core::system::expect_power_off();

    panic!("Power off AND handler failed!");
}

fn welcome_msg() {
    println!("{}", WELCOME_MSG);
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    use tlenix_core::eprintln;

    eprintln!("{} {}", TLENIX_PANIC_TITLE, info);

    // TODO use a better loop
    loop {}
}
