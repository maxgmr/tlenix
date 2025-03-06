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
#![feature(custom_test_frameworks)]
#![test_runner(test_framework::custom_test_runner)]
#![reexport_test_harness_main = "test_main"]

#[cfg(test)]
use core::panic::PanicInfo;

pub mod consts;
pub mod io;
pub mod syscalls;
mod test_framework;

pub use syscalls::SyscallNum;

pub use test_framework::custom_test_runner;

/// Entry point for library tests.
#[cfg(test)]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    #[cfg(test)]
    test_main();

    // TODO exit happily
    #[allow(clippy::empty_loop)]
    loop {}
}

/// Panic handler for library tests.
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    test_framework::test_panic_handler(info)
}
