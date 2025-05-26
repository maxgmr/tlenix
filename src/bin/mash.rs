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

extern crate alloc;

use alloc::{string::String, vec::Vec};
use core::panic::PanicInfo;
use num_enum::TryFromPrimitive;

use tlenix_core::{
    Console, Errno, align_stack_pointer, eprintln, fs, print, println,
    process::{self, ExitStatus},
    system,
};

const MASH_PANIC_TITLE: &str = "mash";

const PROMPT_START: &str = "\u{001b}[94mmash\u{001b}[0m";
const PROMPT_FINISH: &str = "\u{001b}[92;1m:}\u{001b}[0m";

// Used as a backup just in case the current working directory can't be determined.
const CWD_NAME_BACKUP: &str = "?";

// Maximum line size.
const LINE_MAX: usize = 1 << 12;

const HOME_DIR: &str = "/root";

/// Entry point.
///
/// # Panics
///
/// This function panics if the system fails to power off properly.
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    align_stack_pointer!();

    #[cfg(test)]
    process::exit(process::ExitStatus::ExitSuccess);

    // HACK: This stops the compiler from complaining when building the test/debug target
    #[allow(unreachable_code)]
    #[allow(clippy::no_effect)]
    ();

    let console = Console::open().unwrap();
    loop {
        print_prompt();
        let line = console.read_line(LINE_MAX).unwrap();
        let line_string = String::from_utf8_lossy(&line);
        let argv: Vec<&str> = line_string.split_whitespace().collect();

        // Do nothing if nothing was typed
        if argv.is_empty() {
            continue;
        }

        match (argv[0], argv.len()) {
            ("exit", 1) => process::exit(process::ExitStatus::ExitSuccess),
            ("poweroff", 1) => {
                let errno = system::power_off().unwrap_err();
                eprintln!("poweroff fail: {}", errno.as_str());
            }
            ("reboot", 1) => {
                let errno = system::reboot().unwrap_err();
                eprintln!("reboot fail: {}", errno.as_str());
            }
            ("cd", 1) => {
                if let Err(e) = fs::change_dir(HOME_DIR) {
                    eprintln!("{e}");
                }
            }
            ("cd", 2) => {
                if let Err(e) = fs::change_dir(argv[1]) {
                    eprintln!("{e}");
                }
            }
            ("pwd", 1) => match fs::get_cwd() {
                Ok(cwd) => println!("{cwd}"),
                Err(e) => eprintln!("{e}"),
            },
            (_, _) => match process::execute_process(argv, [""; 0].to_vec()) {
                Ok(ExitStatus::ExitFailure(code)) => {
                    if let Ok(errno) = Errno::try_from_primitive(code) {
                        eprintln!("{errno}");
                    } else {
                        eprintln!("Process exited with failure code {code}.");
                    }
                }
                Err(e) => {
                    eprintln!("{e}");
                }
                _ => {}
            },
        }
    }
}

/// Print the MASH shell prompt.
fn print_prompt() {
    let cwd_backup = String::from(CWD_NAME_BACKUP);
    let cwd = fs::get_cwd().unwrap_or(cwd_backup);
    let basename =
        &cwd.rsplit_once('/').map_or(
            cwd.as_str(),
            |(_, last)| if last.is_empty() { "/" } else { last },
        );

    print!("{PROMPT_START} {basename} {PROMPT_FINISH} ");
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    tlenix_core::eprintln!("{} {}", MASH_PANIC_TITLE, info);
    process::exit(process::ExitStatus::ExitFailure(1))
}
