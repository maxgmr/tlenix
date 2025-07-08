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
    EnvVar, Errno, eprintln, parse_argv_envp, print, println,
    process::{self, ExitStatus},
    streams::STDIN,
    term::{
        ControlCharIndex, ControlModeFlags, InputModeFlags, LocalModeFlags, OutputModeFlags,
        SetTermAttrsCmd, Termios,
    },
    try_exit,
};

const PANIC_TITLE: &str = "ted";

/// ANSI escape code to clear the entire screen.
const CLEAR_SCREEN: &str = "\u{001b}[2J";
/// ANSI escape code to move the cursor to the top-left corner.
const CURSOR_TOP_LEFT: &str = "\u{001b}[H";

const READ_MIN_BYTES_READ: u8 = 0;
const READ_MAX_TIME_PASSED: Deciseconds = Deciseconds(1);
const EXIT_CODE: u8 = ctrl_key(b'q');

core::arch::global_asm! {
    ".global _start",
    "_start:",
    "mov rdi, rsp",
    "call start"
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Deciseconds(u8);

/// Get the byte version of "CTRL + this key".
const fn ctrl_key(byte: u8) -> u8 {
    byte & 0x1f
}

/// The current configuration of the editor.
struct Config {
    orig_termios: Termios,
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

/// Enters terminal "raw mode".
///
/// Makes input available character-by-character, disables echo, and disables all special
/// processing of terminal input and output characters.
///
/// More info: [termios(3)](https://www.man7.org/linux/man-pages/man3/termios.3.html)
fn enter_raw_mode() -> Result<(), Errno> {
    STDIN.lock().set_input_mode_flags(
        SetTermAttrsCmd::Tcsetsf,
        InputModeFlags::IGNBRK
            | InputModeFlags::BRKINT
            | InputModeFlags::PARMRK
            | InputModeFlags::ISTRIP
            // | InputModeFlags::INLCR
            // | InputModeFlags::ICRNL
            | InputModeFlags::IXON,
        false,
    )?;
    STDIN
        .lock()
        .set_output_mode_flags(SetTermAttrsCmd::Tcsetsf, OutputModeFlags::OPOST, false)?;
    STDIN.lock().set_local_mode_flags(
        SetTermAttrsCmd::Tcsetsf,
        LocalModeFlags::ECHO
            | LocalModeFlags::ECHONL
            | LocalModeFlags::ICANON
            | LocalModeFlags::ISIG
            | LocalModeFlags::IEXTEN,
        false,
    )?;
    STDIN
        .lock()
        .set_control_mode_flags(SetTermAttrsCmd::Tcsetsf, ControlModeFlags::CSIZE, true)?;
    Ok(())
}

/// Sets the minimum bytes read and maximum time passed before `read` can return.
fn set_read_timeouts(min_bytes_read: u8, max_time_passed: Deciseconds) -> Result<(), Errno> {
    STDIN.lock().set_control_character(
        SetTermAttrsCmd::Tcsetsf,
        ControlCharIndex::Min,
        min_bytes_read,
    )?;
    STDIN.lock().set_control_character(
        SetTermAttrsCmd::Tcsetsf,
        ControlCharIndex::Time,
        max_time_passed.0,
    )
}

/// Clears the screen.
fn clear_screen() {
    print!("{CLEAR_SCREEN}{CURSOR_TOP_LEFT}");
}

/// Restores the terminal to the provided [`Termios`].
fn restore_terminal(termios: &Termios) -> Result<(), Errno> {
    STDIN.lock().set_termios(SetTermAttrsCmd::Tcsetsf, termios)
}

/// Renders the rows of the interface onto the terminal.
fn render_rows() {
    // TODO get actual terminal height
    for _ in 0..24 {
        println!("~");
    }
    print!("{CURSOR_TOP_LEFT}");
}

/// Refreshes the screen, displaying the intended content.
fn refresh_screen() {
    clear_screen();
    render_rows();
}

/// Reads a single keypress from `stdin`.
fn read_keypress() -> Result<u8, Errno> {
    let mut byte_buf = [0];
    // Try to read a byte from stdin.
    loop {
        match STDIN.lock().read(&mut byte_buf) {
            Ok(0) | Err(Errno::Eagain) => {
                // Nothing was read. Try again.
            }
            Ok(_) => {
                // A byte was read. Return the byte.
                return Ok(byte_buf[0]);
            }
            Err(e) => {
                // Non-retryable error. Return the error.
                return Err(e);
            }
        }
    }
}

/// Handles user input, propagating any [`Errno`]s from underlying syscalls. Returns a boolean
/// value dictating whether or not the program should exit.
fn handle_input() -> Result<bool, Errno> {
    let input_byte = read_keypress()?;

    match input_byte {
        EXIT_CODE => {
            // Exit.
            return Ok(true);
        }
        _ => {
            // TODO debug: print input char
            if let Ok(utf8_char) = str::from_utf8(&[input_byte]) {
                tlenix_core::print!("{utf8_char}");
            } else {
                tlenix_core::print!("{:#x}", input_byte);
            }
        }
    }

    Ok(false)
}

fn main(_args: &[String], _env_vars: &[EnvVar]) -> ExitStatus {
    let orig_termios = try_exit!(STDIN.lock().termios());
    let mut config = Config { orig_termios };
    try_exit!(enter_raw_mode());
    try_exit!(set_read_timeouts(READ_MIN_BYTES_READ, READ_MAX_TIME_PASSED));
    clear_screen();

    loop {
        refresh_screen();
        if try_exit!(handle_input()) {
            break;
        }
    }

    try_exit!(restore_terminal(&config.orig_termios));
    clear_screen();
    ExitStatus::ExitSuccess
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    eprintln!("{PANIC_TITLE} {info}");
    process::exit(ExitStatus::ExitFailure(1))
}
