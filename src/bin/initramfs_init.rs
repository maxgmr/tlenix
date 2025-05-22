//! This executable is packed into `initramfs` and executed by the kernel as PID1. It performs the
//! following steps:
//!
//! 1. Mounts essential filesystems
//! 2. Finds and mounts the rootfs
//! 3. Switches to the real rootfs

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

use tlenix_core::{ExitStatus, align_stack_pointer, process, thread};

/// The name of the process for the purposes of the panic message.
const INITRAMFS_INIT_PANIC_TITLE: &str = "initramfs_init";

/// Entry point.
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    align_stack_pointer!();

    #[cfg(test)]
    process::exit(ExitStatus::ExitSuccess);

    // This stops the compiler from complaining when compiling for tests.
    #[allow(unreachable_code)]
    thread::sleep_loop_forever();
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    tlenix_core::eprintln!("{} {}", INITRAMFS_INIT_PANIC_TITLE, info);
    process::exit(ExitStatus::ExitFailure)
}
