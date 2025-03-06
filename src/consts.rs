//! Generally-useful constants.

/// The standard input stream.
pub const STDIN: usize = 0;
/// The standard output stream.
pub const STDOUT: usize = 1;
/// The standard error stream.
pub const STDERR: usize = 2;

/// This is returned if the [reboot](https://man7.org/linux/man-pages/man2/reboot.2.html) syscall
/// fails.
pub const REBOOT_FAIL: i32 = -1;
