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

use tlenix_core::{ExitStatus, align_stack_pointer, print, process::exit};

const MASH_PANIC_TITLE: &str = "mash";

const PROMPT_START: &str = "\u{001b}[94mmash\u{001b}[0m";
const PROMPT_FINISH: &str = "\u{001b}[92;1m:}\u{001b}[0m";

// Used as a backup just in case the current working directory can't be determined.
const CWD_NAME_BACKUP: &str = "?";

// Maximum line size.
const LINE_MAX: usize = 1 << 12;

/// Entry point.
///
/// # Panics
///
/// This function panics if the system fails to power off properly.
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    align_stack_pointer!();

    #[cfg(test)]
    tlenix_core::process::exit(tlenix_core::ExitStatus::ExitSuccess);

    // This stops the compiler from complaining when compiling for tests.
    #[allow(unreachable_code, clippy::never_loop)]
    loop {
        prompt();
        // TODO
        exit(ExitStatus::ExitSuccess);
    }
}

/// Print the MASH shell prompt.
fn prompt() {
    // TODO get CWD name
    let cwd_name = CWD_NAME_BACKUP;

    print!("{PROMPT_START} {cwd_name} {PROMPT_FINISH} ");
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    tlenix_core::eprintln!("{} {}", MASH_PANIC_TITLE, info);
    exit(ExitStatus::ExitFailure)
}
