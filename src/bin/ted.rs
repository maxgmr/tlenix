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
    EnvVar, Errno, eprintln, parse_argv_envp, print,
    process::{self, ExitStatus},
    raw_println,
    streams::STDIN,
    term::{
        ControlCharIndex, ControlModeFlags, InputModeFlags, LocalModeFlags, OutputModeFlags,
        SetTermAttrsCmd, Termios, WinSize,
    },
    try_exit,
};

const PANIC_TITLE: &str = "ted";

const ESC_CODE: u8 = 0x1b;

/// ANSI escape code to clear the entire screen.
const CLEAR_SCREEN: &str = "\u{001b}[2J";
/// ANSI escape code to move the cursor to the top-left corner.
const CURSOR_TOP_LEFT: &str = "\u{001b}[H";
/// ANSI sequence to move the cursor to the bottom-right corner.
const CURSOR_BOTTOM_RIGHT: &str = "\u{001b}[999C\u{001b}[999B";
/// ANSI device status report sequence to get the cursor position.
const GET_CURSOR_POS: &str = "\u{001b}[6n";

const READ_MIN_BYTES_READ: u8 = 0;
const READ_MAX_TIME_PASSED: Deciseconds = Deciseconds(1);
const EXIT_CODE: u8 = ctrl_key(b'q');

const CHECK_TERM_RESPONSE_LIMIT: usize = 64;

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

/// A given row-column point within the screen.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Point {
    row: usize,
    col: usize,
}
impl Point {
    fn try_from_string_helper(value: &str) -> Option<Self> {
        // Format: "[<row>;<col>R"
        let start = value.rfind('[')? + 1;
        let end = value.find('R')?;

        let (first, second) = &value[start..end].split_once(';')?;

        Some(Self {
            row: first.parse().ok()?,
            col: second.parse().ok()?,
        })
    }
}
impl From<Point> for WinSize {
    fn from(value: Point) -> Self {
        Self {
            rows: value.row,
            cols: value.col,
            width: 0,
            height: 0,
        }
    }
}
impl TryFrom<&str> for Point {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from_string_helper(value).ok_or("failed to parse Point from string")
    }
}

/// The current configuration of the editor.
#[derive(Debug, Clone)]
struct Config {
    orig_termios: Termios,
    win_size: WinSize,
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

fn get_win_size() -> WinSize {
    // Try to get window size from system call
    if let Ok(ioctl_win_size) = STDIN.lock().win_size()
        && ioctl_win_size.cols > 0
        && ioctl_win_size.rows > 0
    {
        return ioctl_win_size;
    }

    // Fallback: Move the cursor to the bottom-right and get cursor position
    print!("{CURSOR_BOTTOM_RIGHT}");
    let win_size = if let Ok(pos) = get_cursor_pos() {
        pos.into()
    } else {
        WinSize::default()
    };
    print!("{CURSOR_TOP_LEFT}");
    win_size
}

/// Gets the current position of the cursor on the screen.
fn get_cursor_pos() -> Result<Point, Errno> {
    print!("{GET_CURSOR_POS}");
    let mut buf = [0; CHECK_TERM_RESPONSE_LIMIT];
    let mut stdin = STDIN.lock();

    for byte in &mut buf {
        if stdin.read(core::slice::from_mut(byte))? != 1 || *byte == b'R' {
            break;
        }
    }

    if buf.first() != Some(&ESC_CODE) || buf.get(1) != Some(&b'[') {
        return Err(Errno::Enodata);
    }
    let response = String::from_utf8_lossy(&buf[1..]);
    Point::try_from(response.as_ref()).map_err(|_| Errno::Einval)
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
fn render_rows(config: &Config) {
    let mut render_buf = String::with_capacity(config.win_size.rows * config.win_size.cols);
    for i in 0..config.win_size.rows {
        render_buf.push('~');
        if i < config.win_size.rows - 1 {
            render_buf.push('\r');
            render_buf.push('\n');
        }
    }
    print!("{render_buf}{CURSOR_TOP_LEFT}");
}

/// Refreshes the screen, displaying the intended content.
fn refresh_screen(config: &Config) {
    clear_screen();
    render_rows(config);
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

    try_exit!(enter_raw_mode());
    try_exit!(set_read_timeouts(READ_MIN_BYTES_READ, READ_MAX_TIME_PASSED));
    clear_screen();

    let config = Config {
        orig_termios,
        win_size: get_win_size(),
    };

    loop {
        refresh_screen(&config);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn point_from_str_ok() {
        assert_eq!(
            Point::try_from("^[[12;34R").unwrap(),
            Point { row: 12, col: 34 }
        );
        assert_eq!(
            Point::try_from("\u{001b}[2;547R").unwrap(),
            Point { row: 2, col: 547 }
        );
    }

    #[test_case]
    fn point_from_str_reject_bad() {
        Point::try_from("\u{001b}[;54R").unwrap_err();
        Point::try_from("^[[123;BR").unwrap_err();
        Point::try_from("89;92R").unwrap_err();
        Point::try_from("^[[12;34").unwrap_err();
    }
}
