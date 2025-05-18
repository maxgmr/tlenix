//! Module for the [`OpenOptions`] struct.

use core::default::Default;

use crate::{Errno, fs::OpenFlags};

use super::FileDescriptor;

// Macro to create methods that set open_flags to a given value.
macro_rules! open_flag_setter {
    (
        $(
            $(#[$outer:meta])*
            $method:ident => $flag:ident;
        )*
    ) => {
        $(
            $(#[$outer])*
            pub fn $method(&mut self, value: bool) -> &mut Self {
                self.open_flags.set(OpenFlags::$flag, value);
                self.make_flags_valid(OpenFlags::$flag, value);
                self
            }
        )*
    }
}

/// Used to open a file with a defined set of options and flags. These options determine the
/// behaviour of the opened file.
///
/// Provides functionality analogous to the
/// [standard library's `OpenOptions`](https://doc.rust-lang.org/std/fs/struct.OpenOptions.html#method.open).
///
/// It also self-enforces the flag combinations, meaning that invalid flag combinations won't be
/// possible.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OpenOptions {
    open_flags: OpenFlags,
}
impl OpenOptions {
    /// Creates a new [`OpenOptions`] in read-only mode, with all other options/flags disabled.
    #[must_use]
    pub fn new() -> Self {
        Self {
            open_flags: OpenFlags::empty(),
        }
    }

    /// Sets the read-only flag. When [`Self::open`] is called, the file will be
    /// opened with read-only permissions.
    ///
    /// This is the default behaviour.
    pub fn read_only(&mut self) -> &mut Self {
        self.open_flags
            .remove(OpenFlags::O_RDWR | OpenFlags::O_WRONLY);
        self
    }

    /// Sets the write-only flag. When [`Self::open`] is called, the file will be opened with
    /// write-only permissions.
    pub fn write_only(&mut self) -> &mut Self {
        self.open_flags.remove(OpenFlags::O_RDWR);
        self.open_flags.insert(OpenFlags::O_WRONLY);
        self
    }

    /// Sets the read-write flag. When [`Self::open`] is called, the file will be opened with
    /// both read _and_ write permissions.
    pub fn read_write(&mut self) -> &mut Self {
        self.open_flags.remove(OpenFlags::O_WRONLY);
        self.open_flags.insert(OpenFlags::O_RDWR);
        self
    }

    open_flag_setter!(
        /// If this flag is set, when [`Self::open`] is called, any write operations will start
        /// from the end of the file.
        ///
        /// This flag does nothing if write access is disabled.
        append => O_APPEND;

        /// If this flag is set, when [`Self::open`] is called, the existing file contents will be
        /// truncated to length 0.
        ///
        /// This flag does nothing if write access is disabled.
        truncate => O_TRUNC;

        /// If this flag is set, when [`Self::open`] is called, a new file will be created if the
        /// given file doesn't exist.
        ///
        /// If this flag is reset while [`Self::create_new`] is active, then [`Self::create_new`] will
        /// also be reset. Otherwise, that combo could potentially cause undefined behaviour.
        create => O_CREAT;

        /// If this flag is set, when [`Self::open`] is called, a new file will be created if the
        /// given file doesn't exist, failing with [`Errno::Eexist`] otherwise.
        ///
        /// If this flag is set while [`Self::create`] is disabled, then [`Self::create`] will also
        /// be set. Otherwise, that combo could potentially cause undefined behaviour.
        create_new => O_EXCL;

        /// If this flag is set, when [`Self::open`] is called, cache effects of the I/O to and
        /// from the file will be minimized when possible.
        direct => O_DIRECT;

        /// If this flag is set, when [`Self::open`] is called, the operation will fail if the path
        /// does not lead to a directory.
        directory => O_DIRECTORY;

        /// If this flag is set, when [`Self::open`] is called, the last access time of the file
        /// won't be updated.
        no_update_last_access => O_NOATIME;

        /// If this flag is set, when [`Self::open`] is called and the file is a symbolic link,
        /// then the operation will fail with [`Errno::Eloop`].
        no_follow => O_NOFOLLOW;

        /// If this flag is set, when [`Self::open`] is called, the file itself won't be opened.
        /// Only operations on the file descriptor level will do anything; all others will fail
        /// with [`Errno::Ebadf`].
        path_only => O_PATH;

        /// If this flag is set, when [`Self::open`] is called, write operations on the file will
        /// be done synchronously.
        ///
        /// Put another way, any write operations will only return once all underlying hardware I/O
        /// operations have completed.
        sync => O_SYNC;
    );

    /// Opens the file at the given path with this [`OpenOptions`]' options.
    ///
    /// By default, the file will be opened in read-only mode.
    ///
    /// # Errors
    ///
    /// This function returns an [`Errno`] if the file fails to open for whatever reason.
    pub fn open(&self) -> Result<FileDescriptor, Errno> {
        todo!()
    }

    /// Ensures that any invalid flag combos are remedied.
    fn make_flags_valid(&mut self, last_changed: OpenFlags, value: bool) {
        match (last_changed, value) {
            (OpenFlags::O_CREAT, false) => {
                // O_EXCL without O_CREAT is UB
                self.open_flags.remove(OpenFlags::O_EXCL);
            }
            (OpenFlags::O_EXCL, true) => {
                // O_EXCL without O_CREAT is UB
                self.open_flags.insert(OpenFlags::O_CREAT);
            }
            _ => {}
        }

        // Should be unneeded, but may as well have this just as a final safety check.
        if self
            .open_flags
            .contains(OpenFlags::O_RDWR | OpenFlags::O_WRONLY)
        {
            self.open_flags.remove(OpenFlags::O_WRONLY);
        }
    }
}
impl Default for OpenOptions {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn oo_new() {
        let oo = OpenOptions::new();
        assert_eq!(oo.open_flags, OpenFlags::O_RDONLY);
    }

    #[test_case]
    fn oo_ro() {
        let mut oo = OpenOptions::new();
        oo.write_only().read_only();
        assert_eq!(oo.open_flags, OpenFlags::O_RDONLY);
    }

    #[test_case]
    fn oo_wo() {
        let mut oo = OpenOptions::new();
        oo.write_only();
        assert_eq!(oo.open_flags, OpenFlags::O_WRONLY);
    }

    #[test_case]
    fn oo_wr() {
        let mut oo = OpenOptions::new();
        oo.write_only().read_only().read_write();
        assert_eq!(oo.open_flags, OpenFlags::O_RDWR);
    }

    #[test_case]
    fn no_excl_without_creat() {
        let mut oo = OpenOptions::new();

        oo.create_new(true);
        assert_eq!(
            oo.open_flags,
            OpenFlags::O_RDONLY | OpenFlags::O_EXCL | OpenFlags::O_CREAT
        );

        oo.create(false);
        assert_eq!(oo.open_flags, OpenFlags::O_RDONLY);

        oo.create(true);
        assert_eq!(oo.open_flags, OpenFlags::O_RDONLY | OpenFlags::O_CREAT);

        oo.create_new(true);
        assert_eq!(
            oo.open_flags,
            OpenFlags::O_RDONLY | OpenFlags::O_EXCL | OpenFlags::O_CREAT
        );

        oo.create_new(false);
        assert_eq!(oo.open_flags, OpenFlags::O_RDONLY | OpenFlags::O_CREAT);

        oo.create_new(true);
        assert_eq!(
            oo.open_flags,
            OpenFlags::O_RDONLY | OpenFlags::O_EXCL | OpenFlags::O_CREAT
        );
    }
}
