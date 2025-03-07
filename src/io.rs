//! Functionality related to input and output.

use core::fmt::{Arguments, Write};

use super::{
    SyscallNum,
    consts::{STDERR, STDOUT},
    syscall,
};

/// Print to stdout using format syntax.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (
        $crate::io::__print_str(format_args!($($arg)*))
    );
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
    ($($arg:tt)*) => (
        $crate::io::__print_err(format_args!($($arg)*))
    );
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

/// Represents stderr.
#[derive(Debug)]
struct Stderr;

macro_rules! write_str_impl {
    [$(($t:ty, $fd:expr)),*] => {
        $(impl Write for $t {
            fn write_str(&mut self, s: &str) -> core::fmt::Result {
                // SAFETY: The caller is only able to choose the string itself. Everything else
                // is chosen statically or based on the string length.
                unsafe {
                    syscall!(SyscallNum::Write, $fd, s.as_ptr() as usize, s.len());
                }
                Ok(())
            }
        })*
    };
}
write_str_impl![(Stdout, STDOUT), (Stderr, STDERR)];

/// For [`print`] and [`println`] use only.
#[doc(hidden)]
pub fn __print_str(args: Arguments<'_>) {
    Stdout.write_fmt(args).unwrap();
}

/// For [`eprint`] and [`eprintln`] use only.
#[doc(hidden)]
pub fn __print_err(args: Arguments<'_>) {
    Stderr.write_fmt(args).unwrap();
}

#[cfg(test)]
mod tests;
