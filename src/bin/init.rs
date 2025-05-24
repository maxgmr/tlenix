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

extern crate alloc;

use alloc::vec::Vec;
use core::panic::PanicInfo;

use tlenix_core::{align_stack_pointer, println, process, thread};

const WELCOME_MSG: &str = concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION"));
const TLENIX_PANIC_TITLE: &str = "tlenix";

#[cfg(debug_assertions)]
const SHELL_PATH: &str = "target/x86_64-unknown-linux-none/debug/mash";
#[cfg(not(debug_assertions))]
const SHELL_PATH: &str = "/bin/mash";

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

    // HACK: This stops the compiler from complaining when building the test/debug target
    #[allow(unreachable_code)]
    #[allow(clippy::no_effect)]
    ();

    welcome_msg();

    #[cfg(not(debug_assertions))]
    {
        use tlenix_core::fs;

        // Mount procfs
        if let Err(e) = fs::mount(
            "none",
            "/proc",
            fs::FilesystemType::Proc,
            fs::MountFlags::default(),
        ) {
            panic!("Failed to mount /proc: {}", e);
        }

        // Mount sysfs
        if let Err(e) = fs::mount(
            "none",
            "/sys",
            fs::FilesystemType::Sysfs,
            fs::MountFlags::default(),
        ) {
            panic!("Failed to mount /sys: {}", e);
        }
    }

    // Launch shell with no args
    loop {
        process::execute_process(Vec::from([SHELL_PATH]), Vec::<&'static str>::new()).unwrap();
        println!("Restarting shell...");
        println!("(Enter the \"poweroff\" command to shut down)");
    }
}

fn welcome_msg() {
    println!("{}", WELCOME_MSG);
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    tlenix_core::eprintln!("\u{001b}[91m{} {}\u{001b}[0m", TLENIX_PANIC_TITLE, info);
    thread::sleep_loop_forever();
}
