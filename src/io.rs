//! Functionality related to input and output.

use core::fmt::{Arguments, Write};

use crate::{
    SyscallNum,
    consts::{STDERR, STDOUT},
    syscall,
};

/// Print to stdout using format syntax.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::io::_print_str(format_args!($($arg)*)));
}

/// Print, with a newline, to stdout using format syntax.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

/// Print to stderr using format syntax.
#[macro_export]
macro_rules! eprint {
    ($($arg:tt)*) => ($crate::io::_print_err(format_args!($($arg)*)));
}

/// Print, with a newline, to stderr using format syntax.
#[macro_export]
macro_rules! eprintln {
    () => ($crate::eprint!("\n"));
    ($($arg:tt)*) => ($crate::eprint!("{}\n", format_args!($($arg)*)));
}

/// Represents stdout.
#[derive(Debug)]
struct Stdout;
impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        // SAFETY: The caller is only able to choose the string itself. The syscall number, stream,
        // pointer, and string length are all set properly. The pointer is never written to.
        unsafe {
            print_helper(s, STDOUT);
        }
        Ok(())
    }
}

/// Represents stderr.
#[derive(Debug)]
struct Stderr;
impl Write for Stderr {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        // SAFETY: The caller is only able to choose the string itself. The syscall number, stream,
        // pointer, and string length are all set properly. The pointer is never written to.
        unsafe {
            print_helper(s, STDERR);
        }
        Ok(())
    }
}

/// For [print] and [println] use only.
#[doc(hidden)]
pub fn _print_str(args: Arguments<'_>) {
    Stdout.write_fmt(args).unwrap();
}

/// For [eprint] and [eprintln] use only.
#[doc(hidden)]
pub fn _print_err(args: Arguments<'_>) {
    Stderr.write_fmt(args).unwrap();
}

/// Print the given string to the given file descriptor.
///
/// # Safety
///
/// The caller must ensure the file descriptor is valid!
unsafe fn print_helper(s: &str, fd: usize) {
    unsafe {
        syscall!(SyscallNum::Write, fd, s.as_ptr() as usize, s.len());
    }
}
