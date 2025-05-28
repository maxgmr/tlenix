//! Clears the entire terminal screen.

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

use tlenix_core::{
    eprintln, print,
    process::{self, ExitStatus},
};

const PANIC_TITLE: &str = "clear";

/// ANSI escape code to clear the entire screen.
const CLEAR_SCREEN: &str = "\u{001b}[2J";
/// ANSI escape code to move the cursor to the top-left corner.
const CURSOR_TOP_LEFT: &str = "\u{001b}[H";

core::arch::global_asm! {
    ".global _start",
    "_start:",
    "mov rdi, rsp",
    "call start"
}

/// Clears the entire terminal screen.
///
/// # Safety
///
/// This program must be passed appropriate `execve`-compatible args.
#[unsafe(no_mangle)]
#[allow(unused_variables)]
extern "C" fn start(stack_top: *const usize) -> ! {
    #[cfg(test)]
    process::exit(ExitStatus::ExitSuccess);

    // HACK: This stops the compiler from complaining when building the test/debug target
    #[allow(unreachable_code)]
    #[allow(clippy::no_effect)]
    ();

    let exit_code = main();

    process::exit(exit_code);
}

fn main() -> ExitStatus {
    // Clear the screen and move the cursor to the top-left corner.
    print!("{CLEAR_SCREEN}{CURSOR_TOP_LEFT}");
    ExitStatus::ExitSuccess
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    eprintln!("{PANIC_TITLE} {info}");
    process::exit(ExitStatus::ExitFailure(1))
}
