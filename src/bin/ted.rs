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
use core::{panic::PanicInfo, slice};

use tlenix_core::{
    EnvVar, Errno, eprintln, format, parse_argv_envp, print,
    process::{self, ExitStatus},
    streams::STDIN,
    term::{
        ControlCharIndex, ControlModeFlags, InputModeFlags, LocalModeFlags, OutputModeFlags,
        SetTermAttrsCmd, Termios, WinSize,
    },
    try_exit,
};

const PANIC_TITLE: &str = "ted";

const ESC_CODE: u8 = 0x1b;

const CLEAR_SCREEN: &str = "\u{001b}[2J";
const CLEAR_REMAINING_LINE: &str = "\u{001b}[K";
const CURSOR_TOP_LEFT: &str = "\u{001b}[H";
const CURSOR_BOTTOM_RIGHT: &str = "\u{001b}[999C\u{001b}[999B";
const GET_CURSOR_POS: &str = "\u{001b}[6n";
const HIDE_CURSOR: &str = "\u{001b}[?25l";
const SHOW_CURSOR: &str = "\u{001b}[?25h";

const READ_MIN_BYTES_READ: u8 = 0;
const READ_MAX_TIME_PASSED: Deciseconds = Deciseconds(1);

const CHECK_TERM_RESPONSE_LIMIT: usize = 64;

const KEYPRESS_BUF_LEN: usize = 3;

// Controls
const EXIT_CODE: u8 = ctrl_key(b'q');
const CURSOR_U: u8 = b'k';
const CURSOR_D: u8 = b'j';
const CURSOR_L: u8 = b'h';
const CURSOR_R: u8 = b'l';
const CURSOR_TOP: u8 = b'g';
const CURSOR_BOT: u8 = b'G';
const CURSOR_START: u8 = b'^';
const CURSOR_END: u8 = b'$';

core::arch::global_asm! {
    ".global _start",
    "_start:",
    "mov rdi, rsp",
    "call start"
}

/// Get the byte version of "CTRL + this key".
const fn ctrl_key(byte: u8) -> u8 {
    byte & 0x1f
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Deciseconds(u8);

/// A [`String`] storing all content to be rendered onto the screen every frame.
#[derive(Debug, Clone)]
struct RenderBuffer(String);
impl RenderBuffer {
    fn new(win_size: &WinSize) -> Self {
        RenderBuffer(String::with_capacity(
            // Add a little extra capacity to account for escape codes
            (win_size.rows * win_size.cols) + 20,
        ))
    }
}

/// A given row-column point within the screen.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Point {
    row: usize,
    col: usize,
}
impl Point {
    fn row_bounded_add(&mut self, value: usize, win_size: &WinSize) {
        self.row = Self::bounded_change_helper(self.row.saturating_add(value), win_size.rows - 1);
    }

    fn row_bounded_sub(&mut self, value: usize) {
        self.row = self.row.saturating_sub(value);
    }

    fn col_bounded_add(&mut self, value: usize, win_size: &WinSize) {
        self.col = Self::bounded_change_helper(self.col.saturating_add(value), win_size.cols - 1);
    }

    fn col_bounded_sub(&mut self, value: usize) {
        self.col = self.col.saturating_sub(value);
    }

    fn bounded_change_helper(result: usize, bound: usize) -> usize {
        if result <= bound { result } else { bound }
    }

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
impl From<&Point> for String {
    fn from(value: &Point) -> Self {
        format!("\u{001b}[{};{}H", value.row + 1, value.col + 1)
    }
}

/// The current state of the editor.
#[derive(Debug, Clone)]
struct EditorState {
    orig_termios: Termios,
    win_size: WinSize,
    render_buf: RenderBuffer,
    cursor_pos: Point,
    should_exit: bool,
}
impl EditorState {
    /// Refreshes the screen, rendering the current state of the editor.
    fn refresh_screen(&mut self) {
        let cursor_pos_string: String = (&self.cursor_pos).into();

        self.render_buf.0.clear();

        self.render_buf.0.push_str(HIDE_CURSOR);
        self.render_buf.0.push_str(CURSOR_TOP_LEFT);
        self.add_rows();
        self.render_buf.0.push_str(&cursor_pos_string);
        self.render_buf.0.push_str(SHOW_CURSOR);

        print!("{}", self.render_buf.0);
    }

    /// Adds the interface rows to the render buffer.
    fn add_rows(&mut self) {
        for i in 0..self.win_size.rows {
            self.render_buf.0.push('~');
            self.render_buf.0.push_str(CLEAR_REMAINING_LINE);
            if i < self.win_size.rows - 1 {
                self.render_buf.0.push('\r');
                self.render_buf.0.push('\n');
            }
        }
    }

    /// Handles user input, propagating any [`Errno`]s incurred by underlying syscalls.
    fn handle_input(&mut self) -> Result<(), Errno> {
        let input_byte = read_keypress()?;

        match input_byte {
            EXIT_CODE => {
                self.should_exit = true;
            }
            CURSOR_U | CURSOR_D | CURSOR_L | CURSOR_R => {
                self.move_cursor(input_byte);
            }
            CURSOR_TOP => {
                self.cursor_pos.row = 0;
            }
            CURSOR_BOT => {
                self.cursor_pos.row = self.win_size.rows - 1;
            }
            CURSOR_START => {
                self.cursor_pos.col = 0;
            }
            CURSOR_END => {
                self.cursor_pos.col = self.win_size.cols - 1;
            }
            _ => {}
        }

        Ok(())
    }

    /// Moves the cursor matching the given direction.
    fn move_cursor(&mut self, input: u8) {
        match input {
            CURSOR_U => self.cursor_pos.row_bounded_sub(1),
            CURSOR_D => self.cursor_pos.row_bounded_add(1, &self.win_size),
            CURSOR_L => self.cursor_pos.col_bounded_sub(1),
            CURSOR_R => self.cursor_pos.col_bounded_add(1, &self.win_size),
            _ => {}
        }
    }
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
        if stdin.read(slice::from_mut(byte))? != 1 || *byte == b'R' {
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
            | InputModeFlags::INLCR
            | InputModeFlags::ICRNL
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

/// Reads a single keypress from `stdin`.
fn read_keypress() -> Result<u8, Errno> {
    let mut stdin = STDIN.lock();

    // Try to read a byte from stdin.
    let first_byte = stdin.await_read_byte()?;

    // If it's not an escape code, return it. Otherwise, continue...
    if first_byte != ESC_CODE {
        return Ok(first_byte);
    }

    // Byte is the beginning of an escape sequence. Continue reading.
    let mut seq_buf = [0; KEYPRESS_BUF_LEN];
    if stdin.read(slice::from_mut(&mut seq_buf[0]))? != 1
        || stdin.read(slice::from_mut(&mut seq_buf[1]))? != 1
    {
        // Just the escape code or an incomplete escape sequence was sent. Return.
        return Ok(first_byte);
    }

    stdin.read(slice::from_mut(&mut seq_buf[2]))?;

    // If the char after the escape code _isn't_ `[`, then this isn't an ANSI escape sequence.
    // Return the escape code itself.
    if seq_buf.first() != Some(&b'[') {
        return Ok(first_byte);
    }

    Ok(match (seq_buf.get(1), seq_buf.get(2)) {
        // Bind arrow keys to cursor movements
        (Some(b'A'), _) => CURSOR_U,
        (Some(b'B'), _) => CURSOR_D,
        (Some(b'C'), _) => CURSOR_R,
        (Some(b'D'), _) => CURSOR_L,
        // Bind page up and page down keys to top and bottom of screen
        (Some(b'5'), Some(b'~')) => CURSOR_TOP,
        (Some(b'6'), Some(b'~')) => CURSOR_BOT,
        _ => first_byte,
    })
}

fn main(_args: &[String], _env_vars: &[EnvVar]) -> ExitStatus {
    let orig_termios = try_exit!(STDIN.lock().termios());

    try_exit!(enter_raw_mode());
    try_exit!(set_read_timeouts(READ_MIN_BYTES_READ, READ_MAX_TIME_PASSED));
    clear_screen();

    let win_size = get_win_size();
    let render_buf = RenderBuffer::new(&win_size);
    let mut state = EditorState {
        orig_termios,
        win_size,
        render_buf,
        cursor_pos: Point { row: 0, col: 0 },
        should_exit: false,
    };

    loop {
        state.refresh_screen();
        try_exit!(state.handle_input());

        if state.should_exit {
            break;
        }
    }

    try_exit!(restore_terminal(&state.orig_termios));
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
