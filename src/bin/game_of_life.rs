//! Conway's Game of Life in Tlenix.

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
use core::{panic::PanicInfo, slice};

use tlenix_core::{
    EnvVar, Errno, ansi, ansi_cursor_down, ansi_cursor_right, eprintln, parse_argv_envp, print,
    process::{self, ExitStatus},
    streams::STDIN,
    term::{
        ControlCharIndex, ControlModeFlags, InputModeFlags, LocalModeFlags, OutputModeFlags,
        SetTermAttrsCmd, Termios, WinSize,
    },
    try_exit,
};

const PANIC_TITLE: &str = "hello";

const READ_MIN_BYTES_READ: u8 = 0;
const READ_MAX_TIME_PASSED: Deciseconds = Deciseconds(1);
const CHECK_TERM_RESPONSE_LIMIT: usize = 64;

const ESC_CODE: u8 = 0x1b;

const DEAD_CELL: char = ' ';
const LIVE_CELL: char = '█';
const CURSOR: char = '▒';

core::arch::global_asm! {
    ".global _start",
    "_start:",
    "mov rdi, rsp",
    "call start"
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Deciseconds(u8);

/// A position on the game grid.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
struct Pos {
    row: usize,
    col: usize,
}
impl Pos {
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
impl From<Pos> for WinSize {
    fn from(value: Pos) -> Self {
        Self {
            rows: value.row,
            cols: value.col,
            width: 0,
            height: 0,
        }
    }
}
impl TryFrom<&str> for Pos {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from_string_helper(value).ok_or("failed to parse Pos from string")
    }
}

/// A cell on the game grid.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
enum Cell {
    #[default]
    Dead,
    Alive,
}
impl From<Cell> for char {
    fn from(value: Cell) -> Self {
        match value {
            Cell::Dead => DEAD_CELL,
            Cell::Alive => LIVE_CELL,
        }
    }
}

/// A grid of [`Cell`]s.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Grid {
    dimensions: WinSize,
    cells: Vec<Cell>,
}
impl Grid {
    fn with_dimensions(dimensions: WinSize, cell_type: Cell) -> Self {
        let cells = vec![cell_type; dimensions.rows * dimensions.cols];
        Self { dimensions, cells }
    }

    fn iter(&self) -> impl Iterator<Item = &Cell> {
        self.cells.iter()
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut Cell> {
        self.cells.iter_mut()
    }

    fn rows(&self) -> impl Iterator<Item = &[Cell]> {
        self.cells.chunks(self.dimensions.cols)
    }

    fn rows_mut(&mut self) -> impl Iterator<Item = &mut [Cell]> {
        self.cells.chunks_mut(self.dimensions.cols)
    }

    #[inline]
    fn index(&self, pos: Pos) -> usize {
        (pos.row * self.dimensions.cols) + pos.col
    }

    #[inline]
    fn pos(&self, index: usize) -> Pos {
        let row = index % self.dimensions.cols;
        let col = index / self.dimensions.cols;
        Pos { row, col }
    }

    fn get(&self, pos: Pos) -> Option<&Cell> {
        if !self.in_bounds(pos) {
            return None;
        }
        Some(&self.cells[self.index(pos)])
    }

    fn get_mut(&mut self, pos: Pos) -> Option<&mut Cell> {
        if !self.in_bounds(pos) {
            return None;
        }
        let index = self.index(pos);
        Some(&mut self.cells[index])
    }

    #[inline]
    fn in_bounds(&self, pos: Pos) -> bool {
        pos.row < self.dimensions.rows && pos.col < self.dimensions.cols
    }
}

/// The state of the game.
#[derive(Clone, Debug)]
struct Game {
    orig_termios: Termios,
    prev_grid: Grid,
    grid: Grid,
    render_buf: String,
    should_exit: bool,
    cursor: Pos,
}
impl Game {
    fn start() -> Result<Self, Errno> {
        let orig_termios = STDIN.lock().termios()?;

        enter_raw_mode()?;
        set_read_timeouts(READ_MIN_BYTES_READ, READ_MAX_TIME_PASSED)?;
        clear_screen();
        print!("{}", ansi::ANSI_HIDE_CURSOR);
        let dimensions = get_win_size();
        let render_buf = String::with_capacity(dimensions.rows * dimensions.cols);

        let prev_grid = Grid::with_dimensions(dimensions.clone(), Cell::Dead);
        let grid = Grid::with_dimensions(dimensions, Cell::Dead);

        Ok(Self {
            orig_termios,
            prev_grid,
            grid,
            render_buf,
            should_exit: false,
            cursor: Pos::default(),
        })
    }

    fn render(&mut self) {
        self.render_buf.clear();
        self.render_buf.push_str(ansi::ANSI_CURSOR_TOP_LEFT);

        // Render rows
        for (y, row) in self.grid.rows().enumerate() {
            // No need to do anything if the row hasn't changed
            let prev_row = self.prev_grid.rows().nth(y).unwrap();
            if prev_row == row {
                continue;
            }

            for (x, &cell) in row.iter().enumerate() {
                // No need to do anything if the cell hasn't changed
                if prev_row[x] == cell {
                    continue;
                }

                Self::queue_move_cursor(&mut self.render_buf, x, y);
                // Render the cell
                self.render_buf.push(cell.into());
            }
        }

        // Render cursor
        Self::queue_move_cursor(&mut self.render_buf, self.cursor.col, self.cursor.row);
        self.render_buf.push(CURSOR);

        print!("{}", self.render_buf);
    }

    // Add a cursor move to the render buffer.
    fn queue_move_cursor(render_buf: &mut String, x: usize, y: usize) {
        render_buf.push_str("\u{001b}[");
        render_buf.push_str((y + 1).to_string().as_str());
        render_buf.push(';');
        render_buf.push_str((x + 1).to_string().as_str());
        render_buf.push('H');
    }

    fn handle_input(&mut self) -> Result<(), Errno> {
        let byte = poll_input()?;

        match byte {
            ESC_CODE => self.should_exit = true,
            _ => {}
        }

        Ok(())
    }
}
impl Drop for Game {
    fn drop(&mut self) {
        // Can't handle errors here. Function must be infallible
        let _ = term_restore(&self.orig_termios);
        clear_screen();
    }
}

/// John Horton Conway's [Game of Life](https://en.wikipedia.org/wiki/Conway's_Game_of_Life).
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

fn poll_input() -> Result<u8, Errno> {
    let mut byte_buf = [0];

    // Try to read a byte from stdin.
    loop {
        match STDIN.lock().read(&mut byte_buf) {
            Ok(0) | Err(Errno::Eagain) => {
                // Nothing was read. Try again.
            }
            Ok(_) => {
                // A byte was read. Return it.
                return Ok(byte_buf[0]);
            }
            Err(e) => {
                // Non-retryable error. Return the error.
                return Err(e);
            }
        }
    }
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
fn get_cursor_pos() -> Result<Pos, Errno> {
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
    Pos::try_from(response.as_ref()).map_err(|_| Errno::Einval)
}

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

fn term_restore(orig_termios: &Termios) -> Result<(), Errno> {
    print!("{}", ansi::ANSI_SHOW_CURSOR);
    print!("{}", ansi::ANSI_RESET_GRAPHIC);
    STDIN
        .lock()
        .set_termios(SetTermAttrsCmd::Tcsetsf, orig_termios)
}

/// Clears the screen.
fn clear_screen() {
    print!("{}{}", ansi::ANSI_ERASE_DISPLAY, ansi::ANSI_CURSOR_TOP_LEFT);
}

fn main(_args: &[String], _env_vars: &[EnvVar]) -> ExitStatus {
    let mut game = try_exit!(Game::start());

    loop {
        game.render();
        try_exit!(game.handle_input());

        if game.should_exit {
            break;
        }
    }

    ExitStatus::ExitSuccess
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    // Attempt to restore the terminal as best as one can given the situation
    print!("{}", ansi::ANSI_SHOW_CURSOR);
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
}
