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
#![feature(concat_bytes)]
#![cfg_attr(test, test_runner(tlenix_core::custom_test_runner))]

use core::panic::PanicInfo;

use tlenix_core::{
    data::NullTermStr, nulltermstr, println, process::execute_process, sleep_loop,
    sleep_loop_forever,
};

const WELCOME_MSG: &str = concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION"));
const TLENIX_PANIC_TITLE: &str = "tlenix";

#[cfg(debug_assertions)]
const SHELL_PATH: NullTermStr<44> = nulltermstr!(b"target/x86_64-unknown-linux-none/debug/mash");
#[cfg(not(debug_assertions))]
const SHELL_PATH: NullTermStr<10> = nulltermstr!(b"/bin/mash");

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

    #[cfg(test)]
    tlenix_core::process::exit(tlenix_core::consts::EXIT_SUCCESS);

    #[allow(unreachable_code)]
    welcome_msg();

    // Launch shell
    execute_process(&SHELL_PATH).unwrap();

    sleep_loop().unwrap()
}

fn welcome_msg() {
    println!("{}", WELCOME_MSG);
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    tlenix_core::eprintln!("{} {}", TLENIX_PANIC_TITLE, info);
    sleep_loop_forever()
}
