//! Functionality related to printing strings to streams and formatting them.

use alloc::string::String;
use core::fmt::{Arguments, Write};

use crate::streams::{STDERR, STDOUT};

/// For [`print`] and [`println`] use only.
#[doc(hidden)]
pub fn __print_str(args: Arguments<'_>) {
    #[allow(clippy::unwrap_used)]
    STDOUT.lock().write_fmt(args).unwrap();
}

/// For [`eprint`] and [`eprintln`] use only.
#[doc(hidden)]
pub fn __print_err(args: Arguments<'_>) {
    #[allow(clippy::unwrap_used)]
    STDERR.lock().write_fmt(args).unwrap();
}

/// For [`format`] use only.
#[doc(hidden)]
#[must_use]
pub fn __format(args: Arguments<'_>) -> String {
    let mut buf = String::new();
    #[allow(clippy::unwrap_used)]
    buf.write_fmt(args).unwrap();
    buf
}

/// Creates a [`String`] using interpolation of runtime expressions.
#[macro_export]
macro_rules! format {
    ($($arg:tt)*) => {{$crate::__format(core::format_args!($($arg)*))}};
}

/// Print to the standard output using Rust format syntax.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{$crate::__print_str(core::format_args!($($arg)*))}};
}

/// Print, with a newline, to the standard output using Rust format syntax.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => {{$crate::print!("{}\n", core::format_args!($($arg)*))}};
}

/// Print to the standard error stream using Rust format syntax.
#[macro_export]
macro_rules! eprint {
    ($($arg:tt)*) => {{$crate::__print_err(core::format_args!($($arg)*))}};
}

/// Print, with a newline, to the standard error stream using Rust format syntax.
#[macro_export]
macro_rules! eprintln {
    () => ($crate::eprint!("\n"));
    ($($arg:tt)*) => {{$crate::eprint!("{}\n", core::format_args!($($arg)*))}};
}

#[cfg(test)]
mod tests;
