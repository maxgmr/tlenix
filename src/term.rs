//! Module containing functionality related to the system terminal.

mod ioctl;
mod termios;

// RE-EXPORTS

pub use ioctl::SetTermAttrsCmd;
pub(crate) use ioctl::{get_term_attrs, set_term_attrs};
pub use termios::{
    ControlCharIndex, ControlModeFlags, InputModeFlags, LocalModeFlags, OutputModeFlags, Termios,
};
