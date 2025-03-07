//! Generally-useful constants.

/// The standard input stream.
pub const STDIN: usize = 0;
/// The standard output stream.
pub const STDOUT: usize = 1;
/// The standard error stream.
pub const STDERR: usize = 2;

/// The maximum path length.
pub const PATH_MAX: usize = 0x1000;

/// This is returned by most Linux syscalls if they fail.
pub const SYSCALL_FAIL: i32 = -1;
