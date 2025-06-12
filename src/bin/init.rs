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

use alloc::string::ToString;
use core::panic::PanicInfo;

use tlenix_core::{align_stack_pointer, fs, println, process, thread};

const BACKUP_LOGO: &str = r"  _____ _            _
 |_   _| | ___ _ __ (_)_  __
   | | | |/ _ \ '_ \| \ \/ /
   | | | |  __/ | | | |>  <
   |_| |_|\___|_| |_|_/_/\_\";

const WELCOME_MSG: &str = concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION"));
const TLENIX_PANIC_TITLE: &str = "tlenix";

#[cfg(debug_assertions)]
const SHELL_PATH: &str = "target/x86_64-unknown-linux-none/debug/mash";
#[cfg(not(debug_assertions))]
const SHELL_PATH: &str = "/bin/mash";

#[cfg(debug_assertions)]
const LOGO_PATH: &str = "os_files/etc/initlogo";
#[cfg(not(debug_assertions))]
const LOGO_PATH: &str = "/etc/initlogo";

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
    tlenix_core::process::exit(process::ExitStatus::ExitSuccess);

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
        process::execute_process(&[SHELL_PATH], &[""; 0]).unwrap();
        println!("Restarting shell...");
        #[cfg(not(debug_assertions))]
        println!("(Enter the \"poweroff\" command to shut down)");
        #[cfg(debug_assertions)]
        println!("(Use CTRL+C to exit)");
    }
}

fn welcome_msg() {
    let logo = match fs::OpenOptions::new().open(LOGO_PATH) {
        Ok(file) => file.read_to_string().unwrap_or(BACKUP_LOGO.to_string()),
        Err(_) => BACKUP_LOGO.to_string(),
    };
    println!("\u{001b}[33m{logo}\u{001b}[0m{WELCOME_MSG}");
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    tlenix_core::eprintln!("\u{001b}[91m{} {}\u{001b}[0m", TLENIX_PANIC_TITLE, info);
    thread::sleep_loop_forever();
}
