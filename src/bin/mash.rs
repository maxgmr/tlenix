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

use core::panic::PanicInfo;

use tlenix_core::{
    consts::EXIT_FAILURE, consts::EXIT_SUCCESS, fs::get_current_working_directory, io::Console,
    print, println, process::exit, sleep_loop_forever, system::expect_power_off,
};

const MASH_PANIC_TITLE: &str = "mash";

const PROMPT_START: &str = "\u{001b}[94mMASH\u{001b}[0m";
const PROMPT_FINISH: &str = "\u{001b}[92;1m:}\u{001b}[0m";

const EXIT_BYTES: &[u8] = b"exit\0";
const POWEROFF_BYTES: &[u8] = b"poweroff\0";

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

    println!();

    let console = Console::open_console().unwrap();

    loop {
        prompt();
        let line: [u8; LINE_MAX] = console.read_line().unwrap();
        // Exit if `exit` is typed
        if &line[..5] == EXIT_BYTES {
            exit(EXIT_SUCCESS)
        }
        // Poweroff if `poweroff` is typed
        if &line[..9] == POWEROFF_BYTES {
            expect_power_off();
            exit(EXIT_FAILURE)
        }

        // TODO just echo everything back for now
        if line[0] != 0 {
            if let Ok(utf8_line) = str::from_utf8(&line) {
                println!("{utf8_line}");
            } else {
                println!("UTF-8 error :(");
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
    sleep_loop_forever()
}
