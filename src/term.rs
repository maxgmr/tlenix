//! Module containing functionality related to the system terminal.

mod ioctl;
mod termios;
mod winsize;

// RE-EXPORTS

pub use ioctl::SetTermAttrsCmd;
pub(crate) use ioctl::{get_term_attrs, get_term_size, set_controlling_term, set_term_attrs};
pub use termios::{
    ControlCharIndex, ControlModeFlags, InputModeFlags, LocalModeFlags, OutputModeFlags, Termios,
};
pub use winsize::{DEFAULT_COLS, DEFAULT_ROWS, WinSize};
