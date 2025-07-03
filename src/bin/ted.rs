//! `ted` is a simple text editor.

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
#![cfg_attr(test, reexport_test_harness_main = "test_main")]

extern crate alloc;

use alloc::string::String;
use core::panic::PanicInfo;

use tlenix_core::{
    EnvVar, Errno, eprintln, parse_argv_envp,
    process::{self, ExitStatus},
    streams::STDIN,
    term::{LocalModeFlags, SetTermAttrsCmd},
    try_exit,
};

const PANIC_TITLE: &str = "ted";

core::arch::global_asm! {
    ".global _start",
    "_start:",
    "mov rdi, rsp",
    "call start"
}

/// A simple text editor.
///
/// # Safety
///
/// This program must be passed appropriate `execve`-compatible args.
#[unsafe(no_mangle)]
#[allow(unused_variables)]
unsafe extern "C" fn start(stack_top: *const usize) -> ! {
    #[cfg(test)]
    {
        test_main();
        process::exit(ExitStatus::ExitSuccess);
    }

    // HACK: This stops the compiler from complaining when building the test/debug target
    #[allow(unreachable_code)]
    #[allow(clippy::no_effect)]
    ();

    // SAFETY: This function is being called right at the start of execution before anything else.
    // The stack pointer is retrieved directly from the function args.
    let (argv, envp) = match unsafe { parse_argv_envp(stack_top) } {
        Ok(argv_envp) => argv_envp,
        Err(errno) => process::exit(ExitStatus::ExitFailure(errno as i32)),
    };

    let exit_code = main(&argv, &envp);

    process::exit(exit_code);
}

fn enter_raw_mode() -> Result<(), Errno> {
    // Disable echo and canonical mode
    STDIN.lock().set_local_mode_flags(
        SetTermAttrsCmd::Tcsetsf,
        LocalModeFlags::ECHO | LocalModeFlags::ICANON,
        false,
    )?;
    Ok(())
}

fn exit_raw_mode() -> Result<(), Errno> {
    // Re-enable echo and canonical mode
    STDIN.lock().set_local_mode_flags(
        SetTermAttrsCmd::Tcsetsf,
        LocalModeFlags::ECHO | LocalModeFlags::ICANON,
        true,
    )?;
    Ok(())
}

fn main(_args: &[String], _env_vars: &[EnvVar]) -> ExitStatus {
    try_exit!(enter_raw_mode());

    // Quit the program when `q` is pressed
    let mut current_char: [u8; 1] = [0];
    while (try_exit!(STDIN.lock().read(&mut current_char)) == 1) && (current_char[0] != b'q') {
        // TODO debug
        if let Ok(utf8_char) = str::from_utf8(&current_char) {
            tlenix_core::println!("{utf8_char}");
        } else {
            tlenix_core::println!("{:#x}", current_char[0]);
        }
    }

    try_exit!(exit_raw_mode());

    ExitStatus::ExitSuccess
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    eprintln!("{PANIC_TITLE} {info}");
    process::exit(ExitStatus::ExitFailure(1))
}

#[cfg(test)]
mod tests {
    use super::*;
}
