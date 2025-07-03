//! Module containing functionality related to the system terminal.

mod ioctl;
mod termios;

// RE-EXPORTS

pub(crate) use ioctl::{SetTermAttrsCmd, get_term_attrs, set_term_attrs};
pub use termios::{ControlModeFlags, InputModeFlags, LocalModeFlags, OutputModeFlags, Termios};
