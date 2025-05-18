//! The [`OpenFlags`] bitflags.

bitflags::bitflags! {
    /// All the different flags which can be set for the [open](https://www.man7.org/linux/man-pages/man2/open.2.html)
    /// Linux syscall.
    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct OpenFlags: usize {
        /// File open flag: Open file read-only.
        const O_RDONLY = 0x0;
        /// File open flag: Open file write-only.
        const O_WRONLY = 0x1;
        /// File open flag: Open file read/write.
        const O_RDWR = 0x2;
        /// File open flag: If `path` does not exist, create as regular file.
        const O_CREAT = 0x40;
        /// File open flag: Ensure that this call creates the file. Throw error if file
        /// already exists.
        const O_EXCL = 0x80;
        /// File open flag: If `path` refers to a terminal device, it won't become the process's
        /// controlling terminal.
        const O_NOCTTY = 0x100;
        /// File open flag: If the file already exists and the access mode allows writing, it will
        /// be truncated to length 0.
        const O_TRUNC = 0x200;
        /// File open flag: Open in append mode.
        const O_APPEND = 0x400;
        /// File open flag: Open in nonblocking mode when possible.
        const O_NONBLOCK = 0x800;
        /// File open flag: Open in nonblocking mode when possible.
        const O_NDELAY = 0x800;
        /// File open flag: Write operations on the file will complete according to synchronised
        /// I/O data integrity completion.
        const O_DSYNC = 0x1000;
        /// File open flag: Enable signal-drive I/O.
        const O_ASYNC = 0x2000;
        /// File open flag: Minimise cache effects of the I/O to and from this file.
        const O_DIRECT = 0x4000;
        /// File open flag: If `path` is not a directory, cause the open to fail.
        const O_DIRECTORY = 0x1_0000;
        /// File open flag: Fail if the trailing component of `path` is a symlink.
        const O_NOFOLLOW = 0x2_0000;
        /// File open flag: Don't update the file last access time when the file is read.
        const O_NOATIME = 0x4_0000;
        /// File open flag: Enable close-on-exec for new file descriptor.
        const O_CLOEXEC = 0x8_0000;
        /// File open flag: Write operations on the file will complete according to synchronised
        /// I/O file integrity completion.
        const O_SYNC = 0x10_1000;
        /// File open flag: Obtain a file descriptor without opening the file.
        const O_PATH = 0x20_0000;
    }
}
