//! `mash` = **Ma**x's **Sh**ell. Tlenix's shell program! Provides a command-line user interface
//! for Tlenix.

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

extern crate alloc;

use alloc::{string::String, vec::Vec};
use core::panic::PanicInfo;

use tlenix_core::{
    consts::{EXIT_FAILURE, EXIT_SUCCESS},
    data::NullTermString,
    eprintln,
    fs::get_current_working_directory,
    io::Console,
    print, println,
    process::{execute_process, exit},
    system::{power_off, reboot},
};

const MASH_PANIC_TITLE: &str = "mash";

const PROMPT_START: &str = "\u{001b}[94mmash\u{001b}[0m";
const PROMPT_FINISH: &str = "\u{001b}[92;1m:}\u{001b}[0m";

const LINE_MAX: usize = 1024;

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
    let console = Console::open_console().unwrap();

    println!();

    loop {
        prompt();
        let line = console.read_line_vec(LINE_MAX).unwrap();
        let line_str = String::from_utf8_lossy(&line);
        let argv: Vec<&str> = line_str.split_whitespace().collect();

        // Do nothing if nothing was typed
        if argv.is_empty() {
            continue;
        }

        match (argv[0], argv.len()) {
            ("exit", 1) => exit(EXIT_SUCCESS),
            ("poweroff", 1) => {
                let errno = power_off().unwrap_err();
                eprintln!("poweroff fail: {}", errno.as_str());
            }
            ("reboot", 1) => {
                let errno = reboot().unwrap_err();
                eprintln!("reboot fail: {}", errno.as_str());
            }
            (_, _) => {
                // Create a version of argv compatible with `execve`
                let argv_null_termd: Vec<NullTermString> =
                    argv.iter().map(|&str| NullTermString::from(str)).collect();
                // Execute something and wait for it to finish!
                execute_process(&argv_null_termd).unwrap();
            }
        }
    }
}

/// Print the MASH shell prompt.
fn prompt() {
    // TODO clean this up
    let mut cwd_backup: [u8; LINE_MAX] = [0x00_u8; LINE_MAX];
    cwd_backup[0] = b'?';
    let cwd_str_backup = "?";
    let cwd: &[u8; LINE_MAX] = &get_current_working_directory().unwrap_or(cwd_backup);
    let cwd_str: &str = str::from_utf8(cwd).unwrap_or(cwd_str_backup);
    let cwd_str_trimmed: &str = cwd_str.trim_end_matches('\0');
    let basename: &str = cwd_str_trimmed
        .rsplit_once('/')
        .map_or(
            cwd_str_trimmed,
            |(_, last)| if last.is_empty() { "/" } else { last },
        );

    print!("{PROMPT_START} {basename} {PROMPT_FINISH} ");
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    tlenix_core::eprintln!("{} {}", MASH_PANIC_TITLE, info);
    exit(EXIT_FAILURE)
}
