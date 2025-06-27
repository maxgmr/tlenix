//! Module containing functionality related to the system terminal.

mod ioctl;
mod termios;

// RE-EXPORTS

pub use termios::{ControlModeFlags, InputModeFlags, LocalModeFlags, OutputModeFlags};
