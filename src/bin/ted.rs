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

use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::{fmt::Display, mem, panic::PanicInfo, slice};

use getargs::{Arg, Options};
use tlenix_core::{
    EnvVar, Errno, ansi, ansi_cursor_down, ansi_cursor_down_col1, ansi_cursor_right, eprintln,
    format,
    fs::OpenOptions,
    numbers, parse_argv_envp, print,
    process::{self, ExitStatus},
    streams::STDIN,
    term::{
        ControlCharIndex, ControlModeFlags, InputModeFlags, LocalModeFlags, OutputModeFlags,
        SetTermAttrsCmd, Termios, WinSize,
    },
    time::{GetTimeClock, Timespec, clock_time},
    try_exit,
};

const PANIC_TITLE: &str = "ted";
const STATUS_BAR_TITLE: &str = "TED";
const NEW_FILE_STR: &str = "[new file]";

const ESC_CODE: u8 = 0x1b;

const READ_MIN_BYTES_READ: u8 = 0;
const READ_MAX_TIME_PASSED: Deciseconds = Deciseconds(1);

const CHECK_TERM_RESPONSE_LIMIT: usize = 64;

const KEYPRESS_BUF_LEN: usize = 3;

const TAB_LEN: usize = 4;

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

/// The various command-line options and arguments which can be passed to this program.
#[derive(Clone, Debug, Default)]
struct TedOptions {
    /// The path to the opened file.
    path: Option<String>,
    /// Whether or not to print benchmarks.
    benchmark: bool,
}
impl TryFrom<&[String]> for TedOptions {
    type Error = Errno;

    fn try_from(value: &[String]) -> Result<Self, Self::Error> {
        let mut opts = Options::new(value.iter().map(String::as_str).skip(1));

        let mut ted_options = Self::default();

        while let Some(arg) = opts.next_arg().map_err(|_| Errno::Einval)? {
            match arg {
                Arg::Short('b') | Arg::Long("benchmark" | "bench") => ted_options.benchmark = true,
                Arg::Positional(val) if ted_options.path.is_none() => {
                    ted_options.path = Some(val.to_string());
                }
                _ => {}
            }
        }

        Ok(ted_options)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum Key {
    Ascii(u8),
    UpArrow,
    DownArrow,
    RightArrow,
    LeftArrow,
    PageUp,
    PageDown,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Home,
    End,
    Insert,
    Delete,
}
impl Key {
    /// Attempts to match the given escape sequence to a [`Key`] variant.
    fn try_from_esc(seq: [u8; KEYPRESS_BUF_LEN]) -> Option<Self> {
        match (seq.get(1), seq.get(2)) {
            (Some(b'A'), _) => Some(Self::UpArrow),
            (Some(b'B'), _) => Some(Self::DownArrow),
            (Some(b'C'), _) => Some(Self::RightArrow),
            (Some(b'D'), _) => Some(Self::LeftArrow),
            (Some(b'5'), Some(b'~')) => Some(Self::PageUp),
            (Some(b'6'), Some(b'~')) => Some(Self::PageDown),
            (Some(b'1'), Some(b'5')) => Some(Self::F5),
            (Some(b'1'), Some(b'7')) => Some(Self::F6),
            (Some(b'1'), Some(b'8')) => Some(Self::F7),
            (Some(b'1'), Some(b'9')) => Some(Self::F8),
            (Some(b'2'), Some(b'0')) => Some(Self::F9),
            (Some(b'2'), Some(b'1')) => Some(Self::F10),
            (Some(b'2'), Some(b'3')) => Some(Self::F11),
            (Some(b'2'), Some(b'4')) => Some(Self::F12),
            (Some(b'H'), _) => Some(Self::Home),
            (Some(b'F'), _) => Some(Self::End),
            (Some(b'2'), Some(b'~')) => Some(Self::Insert),
            (Some(b'3'), Some(b'~')) => Some(Self::Delete),
            // // DEBUG ONLY
            // _ => {
            //     clear_screen();
            //     print!(
            //         "{}",
            //         String::from_utf8(seq.to_vec()).unwrap()
            //     );
            //     tlenix_core::thread::sleep(&core::time::Duration::from_secs(1)).unwrap();
            //     None
            // }
            _ => None,
        }
    }
}
impl From<u8> for Key {
    fn from(value: u8) -> Self {
        Self::Ascii(value)
    }
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
impl Display for RenderBuffer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut formatted =
            String::with_capacity(self.0.len() + self.0.matches('\t').count() * (TAB_LEN - 1));
        for c in self.0.chars() {
            if c == '\t' {
                for _ in 0..TAB_LEN {
                    formatted.push(' ');
                }
            } else {
                formatted.push(c);
            }
        }
        write!(f, "{formatted}")
    }
}

/// The cursor position within the document.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Cursor(Point);
impl Cursor {
    /// Converts this position to the cursor position on the visible screen, then produces the
    /// terminal sequence which moves the terminal cursor to that position.
    fn term_seq(&self, row_offset: usize, col_offset: usize) -> String {
        format!(
            "\u{001b}[{};{}H",
            self.0.row.saturating_sub(row_offset) + 1,
            self.0.col.saturating_sub(col_offset) + 1
        )
    }
}

/// A given row-column point within the screen.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
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
impl From<&Point> for String {
    fn from(value: &Point) -> Self {
        format!("\u{001b}[{};{}H", value.row + 1, value.col + 1)
    }
}

/// The different types of elements which can be rendered as part of the [`StatusBar`].
///
/// [`StatusBarElem::Code`]s don't take up space, while [`StatusBarElem::Text`]s and
/// [`StatusBarElem::Char`]s _do_.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum StatusBarElem<'a> {
    Code(&'a str),
    Text(&'a str),
    Char(char),
}
impl StatusBarElem<'_> {
    fn len(&self) -> usize {
        match self {
            Self::Code(_) => 0,
            Self::Text(t) => t.len(),
            Self::Char(_) => 1,
        }
    }
}

/// The editor's status bar, displaying information about the current file.
#[derive(Debug, Clone)]
struct StatusBar {
    /// The path of the currently-open file.
    file_path: String,
    /// The last frame time benchmark, if any.
    last_frame_time: Option<Timespec>,
    /// The rendered form of the status bar.
    rendered: String,
}
impl StatusBar {
    fn new(file_path: &str, cols: usize) -> Self {
        let mut status_bar = Self {
            file_path: file_path.to_string(),
            last_frame_time: None,
            rendered: String::new(),
        };
        status_bar.update_render(cols);
        status_bar
    }

    // /// Updates [`Self::file_path`].
    // fn update_file(&mut self, file: &str, cols: usize) {
    //     self.file_path = file.to_string();
    //     self.update_render(cols);
    // }

    /// Updates [`Self::last_frame_time`].
    fn update_benchmark(&mut self, last_frame_time: Timespec, cols: usize) {
        self.last_frame_time = Some(last_frame_time);
        self.update_render(cols);
    }

    /// Updates the rendered state of this [`StatusBar`]. Must be called every time the state is
    /// changed in any way.
    fn update_render(&mut self, cols: usize) {
        use StatusBarElem::{Char, Code, Text};

        self.rendered.clear();
        self.rendered.push_str(ansi::ANSI_INVERT);

        let mut elems = vec![
            Code(ansi::ANSI_FG_BLUE),
            Char(' '),
            Text(STATUS_BAR_TITLE),
            Char(' '),
            Code(ansi::ANSI_FG_GREEN),
            Char(' '),
            Text(&self.file_path),
            Char(' '),
            Code(ansi::ANSI_FG_DEFAULT),
        ];

        let formatted_lft = self
            .last_frame_time
            .map_or(String::new(), |lft| format!("{lft}"));
        if !formatted_lft.is_empty() {
            elems.push(Text("Last frame time: "));
            elems.push(Text(&formatted_lft));
        }

        let mut elems_len = 0;
        for elem in elems {
            // Don't add the next element if it would make the status bar string too long
            if (elems_len + elem.len()) > cols {
                break;
            }

            match elem {
                Code(c) => self.rendered.push_str(c),
                Text(t) => self.rendered.push_str(t),
                Char(c) => self.rendered.push(c),
            }
            elems_len += elem.len();
        }

        // Fill the remaining space with inverted spaces
        while elems_len < cols {
            self.rendered.push(' ');
            elems_len += 1;
        }

        self.rendered.push_str(ansi::ANSI_INVERT_OFF);
    }
}
impl Display for StatusBar {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.rendered)
    }
}

/// The current state of the editor.
#[derive(Debug, Clone)]
struct EditorState {
    /// The original state of the terminal.
    orig_termios: Termios,
    /// The options and parameters of this editor.
    options: TedOptions,
    /// The size of the terminal window (minus the status bar).
    win_size: WinSize,
    /// The individual lines of the text currently being edited.
    editor_rows: Vec<String>,
    /// The previous frame's individual lines of the text currently being edited.
    editor_rows_prev: Vec<String>,
    /// The status bar of the editor.
    status_bar: StatusBar,
    /// Wrapper around a [`String`]. This [`String`] is generated and rendered to the screen every
    /// frame.
    render_buf: RenderBuffer,
    /// The position of the cursor within the text being edited.
    cursor: Cursor,
    /// The offset of the screen relative to the start of the file.
    screen_offset: Point,
    /// Whether or not the program should exit on the next frame.
    should_exit: bool,
}
impl EditorState {
    /// Start up the editor, setting up the terminal accordingly. The terminal is returned to its
    /// previous state when this [`EditorState`] is dropped.
    fn start(options: TedOptions) -> Result<Self, Errno> {
        let orig_termios = STDIN.lock().termios()?;

        enter_raw_mode()?;
        set_read_timeouts(READ_MIN_BYTES_READ, READ_MAX_TIME_PASSED)?;
        clear_screen();

        let mut win_size = get_win_size();
        // Shrink the window height by 1 to make room for status bar
        win_size.rows -= 1;
        // Set the scrolling region to the top and bottom row.
        print!("\u{001b}[{};{}r", 1, win_size.rows);

        // Read the file (if provided)
        let editor_rows = if let Some(path) = &options.path {
            let mut er = Vec::new();
            for line in OpenOptions::new()
                .create(true)
                .open(path)?
                .read_to_string()?
                .lines()
            {
                er.push(line.to_string());
            }
            er
        } else {
            let mut er = Vec::with_capacity(win_size.rows);
            er.push(String::new());
            er
        };
        let editor_rows_prev = editor_rows.clone();

        let render_buf = RenderBuffer::new(&win_size);
        let cols = win_size.cols;

        let sb_file = options.path.clone().unwrap_or(NEW_FILE_STR.to_string());

        let mut result = Self {
            orig_termios,
            options,
            win_size,
            editor_rows,
            editor_rows_prev,
            render_buf,
            status_bar: StatusBar::new(&sb_file, cols),
            cursor: Cursor(Point::default()),
            screen_offset: Point::default(),
            should_exit: false,
        };
        result.cursor_up(usize::MAX);
        result.cursor_left(usize::MAX);
        Ok(result)
    }

    /// Refreshes the screen, rendering the current state of the editor.
    fn refresh_screen(&mut self) {
        let start_time = if self.options.benchmark {
            clock_time(GetTimeClock::Monotonic).unwrap()
        } else {
            Timespec::default()
        };

        self.render_buf.0.clear();

        self.render_buf.0.push_str(ansi::ANSI_HIDE_CURSOR);
        self.render_buf.0.push_str(ansi::ANSI_CURSOR_TOP_LEFT);

        self.add_rows();

        // Move cursor to bottom left of screen in order to render status bar
        self.render_buf.0.push_str(ansi_cursor_down_col1!(999));
        self.add_status_bar();

        // Move the cursor to its "actual" position on the screen
        self.render_buf.0.push_str(
            &self
                .cursor
                .term_seq(self.screen_offset.row, self.screen_offset.col),
        );
        self.render_buf.0.push_str(ansi::ANSI_SHOW_CURSOR);

        // Render this frame
        print!("{}", self.render_buf);

        if self.options.benchmark {
            self.status_bar.update_benchmark(
                clock_time(GetTimeClock::Monotonic).unwrap() - start_time,
                self.win_size.cols,
            );
            // Re-draw status bar now that benchmarking is done, then move cursor back to proper
            // place
            print!(
                "{}{}{}",
                ansi_cursor_down_col1!(999),
                self.status_bar.rendered,
                self.cursor
                    .term_seq(self.screen_offset.row, self.screen_offset.col),
            );
        }
    }

    /// Adds the interface rows to the render buffer.
    fn add_rows(&mut self) {
        let row_start = self.screen_offset.row;
        let row_finish = self.win_size.rows + self.screen_offset.row;

        // Account for ANSI console codes in length
        let mut new_line = String::with_capacity(self.win_size.cols * 2);

        for i in row_start..row_finish {
            new_line.clear();

            let index_width = self.col_lower_bound() - 1;
            if let Some(line) = self.editor_rows.get(i) {
                let line_num = format!("{:>index_width$} ", i + 1);
                new_line.push_str(ansi::ANSI_FG_B_BLACK);
                new_line.push_str(&line_num);
                new_line.push_str(ansi::ANSI_RESET_GRAPHIC);
                new_line.push_str(self.visible_row_slice(line.as_ref()));
            } else {
                // Empty line
                new_line.push_str(ansi::ANSI_FG_B_BLACK);
                new_line.push('~');
                new_line.push(' ');
                new_line.push_str(ansi::ANSI_RESET_GRAPHIC);
            }

            new_line.push_str(ansi::ANSI_ERASE_REMAINING_LINE);

            let screen_row_index = i - row_start;
            let row_changed = self.editor_rows_prev.get(screen_row_index) != Some(&new_line);

            if row_changed {
                // Move cursor to screen index of row
                self.render_buf.0.push_str("\u{001b}[");
                self.render_buf
                    .0
                    .push_str((screen_row_index + 1).to_string().as_str());
                self.render_buf.0.push_str(";1H");
                // Draw line
                self.render_buf.0.push_str(&new_line);

                // Update `Self::editor_rows_prev`
                if screen_row_index < self.editor_rows_prev.len() {
                    self.editor_rows_prev[screen_row_index] = mem::take(&mut new_line);
                } else {
                    // Draw row
                    self.editor_rows_prev.push(mem::take(&mut new_line));
                }
            }
        }

        // Truncate extra lines if the previous frame had more
        if self.editor_rows_prev.len() > self.win_size.rows {
            for i in self.win_size.rows..self.editor_rows_prev.len() {
                // Move to line and clear it
                self.render_buf.0.push_str("\u{001b}[");
                self.render_buf.0.push_str((i + 1).to_string().as_str());
                self.render_buf.0.push_str(";1H\u{001b}[K");
            }
            self.editor_rows_prev.truncate(self.win_size.rows);
        }
    }

    /// Adds the status bar to the render buffer.
    fn add_status_bar(&mut self) {
        self.render_buf.0.push_str(&self.status_bar.rendered);
    }

    /// Gets the slice of the editor row which is visible on the screen.
    fn visible_row_slice<'a>(&self, current_row: &'a str) -> &'a str {
        if current_row.is_empty() {
            return "";
        }
        let slice_start = self.screen_offset.col.clamp(0, current_row.len() - 1);
        let slice_end = (self.win_size.cols + self.screen_offset.col).clamp(0, current_row.len());
        if slice_start >= slice_end {
            return "";
        }

        &current_row[slice_start..slice_end]
    }

    /// Handles user input, propagating any [`Errno`]s incurred by underlying syscalls.
    fn handle_input(&mut self) -> Result<(), Errno> {
        use Key::Ascii;

        let keypress = read_keypress()?;

        match keypress {
            Ascii(EXIT_CODE) => {
                self.should_exit = true;
            }
            Ascii(CURSOR_U) | Key::UpArrow => self.cursor_up(1),
            Ascii(CURSOR_TOP) | Key::PageUp => self.cursor_up(usize::MAX),
            Ascii(CURSOR_D) | Key::DownArrow => self.cursor_down(1),
            Ascii(CURSOR_BOT) | Key::PageDown => self.cursor_down(usize::MAX),
            Ascii(CURSOR_L) | Key::LeftArrow => self.cursor_left(1),
            Ascii(CURSOR_START) | Key::Home => self.cursor_left(usize::MAX),
            Ascii(CURSOR_R) | Key::RightArrow => self.cursor_right(1),
            Ascii(CURSOR_END) | Key::End => self.cursor_right(usize::MAX),
            _ => {}
        }

        Ok(())
    }

    fn cursor_left(&mut self, amount: usize) {
        self.cursor.0.col = self
            .cursor
            .0
            .col
            .saturating_sub(amount)
            .clamp(self.col_lower_bound(), usize::MAX);

        self.scroll();
    }

    fn cursor_right(&mut self, amount: usize) {
        let upper_bound = (self.col_lower_bound() + core::cmp::max(self.current_line_len(), 1)) - 1;
        self.cursor.0.col = self
            .cursor
            .0
            .col
            .saturating_add(amount)
            .clamp(0, upper_bound);

        self.scroll();
    }

    fn cursor_up(&mut self, amount: usize) {
        self.cursor.0.row = self
            .cursor
            .0
            .row
            .saturating_sub(amount)
            .clamp(0, usize::MAX);

        self.cursor_right(0);
        self.scroll();
    }

    fn cursor_down(&mut self, amount: usize) {
        self.cursor.0.row = self
            .cursor
            .0
            .row
            .saturating_add(amount)
            .clamp(0, self.editor_rows.len() - 1);

        self.cursor_right(0);
        self.scroll();
    }

    /// Moves the screen offset to accomodate the new cursor position (if necessary).
    fn scroll(&mut self) {
        // Handle vertical scrolling
        if self.cursor.0.row < self.screen_offset.row {
            let diff = self.screen_offset.row - self.cursor.0.row;
            let len = self.editor_rows_prev.len();
            // Move screen up
            self.screen_offset.row = self.cursor.0.row;
            // Optimize next frame- shift existing rows
            self.editor_rows_prev.rotate_right(diff % len);
            // Scroll rows up on terminal
            print!("\u{001b}[{}T", diff);
        } else if self.cursor.0.row >= (self.screen_offset.row + self.win_size.rows) {
            let diff = self.cursor.0.row - ((self.screen_offset.row + self.win_size.rows) - 1);
            let len = self.editor_rows_prev.len();
            // Move screen down
            self.screen_offset.row = (self.cursor.0.row - self.win_size.rows) + 1;
            // Optimize next frame- shift existing rows
            self.editor_rows_prev.rotate_left(diff % len);
            // Scroll rows up down terminal
            print!("\u{001b}[{}S", diff);
        }

        // Handle horizontal scrolling
        if self.cursor.0.col < self.screen_offset.col {
            // Move screen left
            self.screen_offset.col = self.cursor.0.col;
        } else if self.cursor.0.col >= (self.screen_offset.col + self.win_size.cols) {
            // Move screen right
            self.screen_offset.col = (self.cursor.0.col - self.win_size.cols) + 1;
        }
    }

    /// Gets the lower bound of the cursor X-coordinate.
    fn col_lower_bound(&self) -> usize {
        numbers::num_digits_base10(self.editor_rows.len()) + 1
    }

    /// Gets the current editor line.
    fn current_line(&self) -> &str {
        self.editor_rows
            .get(self.cursor.0.row)
            .map_or("", String::as_ref)
    }

    /// Gets the length of the current editor line.
    fn current_line_len(&self) -> usize {
        self.current_line().len()
    }
}
impl Drop for EditorState {
    fn drop(&mut self) {
        // `Self::drop` has to succeed- we unfortunately can't check to see whether or not this was
        // successful :(
        let _ = restore_terminal(&self.orig_termios);
        clear_screen();
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
    print!("{}{}", ansi_cursor_down!(999), ansi_cursor_right!(999));
    let win_size = if let Ok(pos) = get_cursor_pos() {
        pos.into()
    } else {
        WinSize::default()
    };
    print!("{}", ansi::ANSI_CURSOR_TOP_LEFT);
    win_size
}

/// Gets the current position of the cursor on the screen.
fn get_cursor_pos() -> Result<Point, Errno> {
    print!("{}", ansi::ANSI_GET_CURSOR_POS);
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
    print!("{}{}", ansi::ANSI_ERASE_DISPLAY, ansi::ANSI_CURSOR_TOP_LEFT);
}

/// Restores the terminal to the provided [`Termios`].
fn restore_terminal(termios: &Termios) -> Result<(), Errno> {
    STDIN.lock().set_termios(SetTermAttrsCmd::Tcsetsf, termios)
}

/// Reads a single keypress from `stdin`.
fn read_keypress() -> Result<Key, Errno> {
    let mut stdin = STDIN.lock();

    // Try to read a byte from stdin.
    let first_byte = stdin.await_read_byte()?;

    // If it's not an escape code, return it. Otherwise, continue...
    if first_byte != ESC_CODE {
        return Ok(first_byte.into());
    }

    // Byte is the beginning of an escape sequence. Continue reading.
    let mut seq_buf = [0; KEYPRESS_BUF_LEN];
    if stdin.read(slice::from_mut(&mut seq_buf[0]))? != 1
        || stdin.read(slice::from_mut(&mut seq_buf[1]))? != 1
    {
        // Just the escape code or an incomplete escape sequence was sent. Return.
        return Ok(first_byte.into());
    }

    stdin.read(slice::from_mut(&mut seq_buf[2]))?;

    // If the char after the escape code _isn't_ `[`, then this isn't an ANSI escape sequence.
    // Return the escape code itself.
    if seq_buf.first() != Some(&b'[') {
        return Ok(first_byte.into());
    }

    Ok(Key::try_from_esc(seq_buf).unwrap_or(first_byte.into()))
}

fn main(args: &[String], _env_vars: &[EnvVar]) -> ExitStatus {
    let ted_options = try_exit!(TedOptions::try_from(args));
    let mut state = try_exit!(EditorState::start(ted_options));

    loop {
        state.refresh_screen();
        try_exit!(state.handle_input());

        if state.should_exit {
            break;
        }
    }

    ExitStatus::ExitSuccess
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    // Attempt to restore the terminal as best as one can given the situation
    print!("{}", ansi::ANSI_RESET_GRAPHIC);
    let _ = STDIN.lock().set_input_mode_flags(
        SetTermAttrsCmd::Tcsetsf,
        InputModeFlags::IGNBRK
            | InputModeFlags::BRKINT
            | InputModeFlags::PARMRK
            | InputModeFlags::ISTRIP
            | InputModeFlags::INLCR
            | InputModeFlags::ICRNL
            | InputModeFlags::IXON,
        true,
    );
    let _ =
        STDIN
            .lock()
            .set_output_mode_flags(SetTermAttrsCmd::Tcsetsf, OutputModeFlags::OPOST, true);
    let _ = STDIN.lock().set_local_mode_flags(
        SetTermAttrsCmd::Tcsetsf,
        LocalModeFlags::ECHO
            | LocalModeFlags::ECHONL
            | LocalModeFlags::ICANON
            | LocalModeFlags::ISIG
            | LocalModeFlags::IEXTEN,
        true,
    );
    let _ = STDIN.lock().set_control_mode_flags(
        SetTermAttrsCmd::Tcsetsf,
        ControlModeFlags::CSIZE,
        false,
    );
    clear_screen();
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
