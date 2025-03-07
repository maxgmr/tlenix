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

/// File open flag: Open file read-only.
pub const O_RDONLY: usize = 0x0;
/// File open flag: Open file write-only.
pub const O_WRONLY: usize = 0x1;
/// File open flag: Open file read/write.
pub const O_RDWR: usize = 0x2;
/// File open flag: If `path` does not exist, create as regular file.
pub const O_CREAT: usize = 0x40;
/// File open flag: Ensure that this call creates the file. Throw error if file already exists.
pub const O_EXCL: usize = 0x80;
/// File open flag: If `path` refers to a terminal device, it won't become the process's controlling
/// terminal.
pub const O_NOCTTY: usize = 0x100;
/// File open flag: If the file already exists and the access mode allows writing, it will be
/// truncated to length 0.
pub const O_TRUNC: usize = 0x200;
/// File open flag: Open in append mode.
pub const O_APPEND: usize = 0x400;
/// File open flag: Open in nonblocking mode when possible.
pub const O_NONBLOCK: usize = 0x800;
/// File open flag: Open in nonblocking mode when possible.
pub const O_NDELAY: usize = 0x800;
/// File open flag: Write operations on the file will complete according to synchronised I/O data
/// integrity completion.
pub const O_DSYNC: usize = 0x1000;
/// File open flag: Enable signal-drive I/O.
pub const O_ASYNC: usize = 0x2000;
/// File open flag: Minimise cache effects of the I/O to and from this file.
pub const O_DIRECT: usize = 0x4000;
/// File open flag: If `path` is not a directory, cause the open to fail.
pub const O_DIRECTORY: usize = 0x1_0000;
/// File open flag: Fail if the trailing component of `path` is a symlink.
pub const O_NOFOLLOW: usize = 0x2_0000;
/// File open flag: Don't update the file last access time when the file is read.
pub const O_NOATIME: usize = 0x4_0000;
/// File open flag: Enable close-on-exec for new file descriptor.
pub const O_CLOEXEC: usize = 0x8_0000;
/// File open flag: Write operations on the file will complete according to synchronised I/O file
/// integrity completion.
pub const O_SYNC: usize = 0x10_1000;
/// File open flag: Obtain a file descriptor without opening the file.
pub const O_PATH: usize = 0x20_0000;
