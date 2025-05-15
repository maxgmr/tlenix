//! The `init` program for `tlenix`. Expected location is at `/sbin/init` so the Linux kernel can
//! call it after boot.

#![warn(
    missing_docs,
    missing_debug_implementations,
    rust_2018_idioms,
    clippy::all,
    clippy::pedantic
)]
#![feature(concat_bytes)]
#![no_std]
#![no_main]

use core::panic::PanicInfo;

use tlenix_core::align_stack_pointer;

/// Entry point.
///
/// # Panics
///
/// This function panics if the system fails to power off properly. This is intended behaviour for
/// a Linux-based init program.
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    align_stack_pointer!();

    // TODO replace with better loop
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    // TODO
    loop {}
}
