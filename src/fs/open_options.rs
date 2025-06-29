//! Module for the [`OpenOptions`] struct.

use core::default::Default;

use crate::{
    Errno, NixString, SyscallNum,
    fs::{File, FilePermissions, OpenFlags},
    syscall_result,
};

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

// Macro to create methods that set file_permissions to a given value.
macro_rules! file_permissions_setter {
    (
        $(
            $(#[$outer:meta])*
            $method:ident => $flag:ident;
        )*
    ) => {
       $(
           $(#[$outer])*
           pub fn $method(&mut self, value: bool) -> &mut Self {
               self.file_permissions.set(FilePermissions::$flag, value);
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
    file_permissions: FilePermissions,
}
impl OpenOptions {
    /// Defines a dummy [`OpenOptions`] in cases where it doesn't matter; e.g., when accessing the
    /// standard streams.
    #[doc(hidden)]
    #[must_use]
    pub(crate) const fn dummy() -> Self {
        Self {
            open_flags: OpenFlags::empty(),
            file_permissions: FilePermissions::empty(),
        }
    }
    /// Creates a new [`OpenOptions`] in read-only and close-on-exec mode, with all other
    /// options/flags disabled.
    ///
    /// File permissions only apply to newly-created files.
    ///
    /// File permissions are, by default, set to 0644 (owner can read and write, group and others
    /// can read).
    ///
    /// By default, immediately calling [`Self::open`] will open the file in read-only mode.
    #[must_use]
    pub fn new() -> Self {
        Self {
            open_flags: OpenFlags::default(),
            file_permissions: FilePermissions::default(),
        }
    }

    /// Opens the [`File`] at the given path with this [`OpenOptions`]' options. Utilizes the
    /// [`open`](https://www.man7.org/linux/man-pages/man2/open.2.html) Linux syscall.
    ///
    /// By default, the file will be opened in read-only mode.
    ///
    /// # Errors
    ///
    /// This function returns an [`Errno`] if the file fails to open for whatever reason. These
    /// errors are propagated up from the underlying `open` syscall.
    pub fn open<NS: Into<NixString>>(&self, path: NS) -> Result<File, Errno> {
        let path_str: NixString = path.into();
        let file_descriptor = unsafe {
            syscall_result!(
                SyscallNum::Open,
                path_str.as_ptr(),
                self.open_flags.bits(),
                self.file_permissions.bits()
            )?
        };
        Ok(File::__new(file_descriptor.into(), self))
    }

    /// Sets the read-only flag. When [`Self::open`] is called, the file will be
    /// opened with read-only permissions.
    ///
    /// This is the default behaviour.
    ///
    /// Setting the read-only flag will disable [`Self::truncate`] if it was enabled, as read-only
    /// + truncate is undefined behaviour.
    ///
    /// Setting the read-only flag will also disable [`Self::create_temp`] if it was enabled, as
    /// opening a tempfile in read-only mode is forbidden.
    pub fn read_only(&mut self) -> &mut Self {
        self.open_flags
            .remove(OpenFlags::O_RDWR | OpenFlags::O_WRONLY);
        self.make_flags_valid(OpenFlags::O_RDONLY, true);
        self
    }

    /// Sets the write-only flag. When [`Self::open`] is called, the file will be opened with
    /// write-only permissions.
    pub fn write_only(&mut self) -> &mut Self {
        self.open_flags.remove(OpenFlags::O_RDWR);
        self.open_flags.insert(OpenFlags::O_WRONLY);
        self.make_flags_valid(OpenFlags::O_WRONLY, true);
        self
    }

    /// Sets the read-write flag. When [`Self::open`] is called, the file will be opened with
    /// both read _and_ write permissions.
    pub fn read_write(&mut self) -> &mut Self {
        self.open_flags.remove(OpenFlags::O_WRONLY);
        self.open_flags.insert(OpenFlags::O_RDWR);
        self.make_flags_valid(OpenFlags::O_RDWR, true);
        self
    }

    /// Sets the file mode to the given [`FilePermissions`]. Will overwrite any existing file
    /// permissions.
    pub fn set_mode<FP: Into<FilePermissions>>(&mut self, mode: FP) -> &mut Self {
        self.file_permissions = mode.into();
        self
    }

    /// Wrapper around the [`bitflags::Flags::contains`] function for this [`OpenOptions`]'
    /// [`OpenFlags`].
    #[must_use]
    pub fn flags_contains(&self, open_flags: OpenFlags) -> bool {
        self.open_flags.contains(open_flags)
    }

    open_flag_setter!(
        /// If this flag is set, when [`Self::open`] is called, any write operations will start
        /// from the end of the file.
        ///
        /// This flag does nothing if write access is disabled.
        append => O_APPEND;

        /// If this flag is set, then [`Self::open`] is called, the resulting file won't be
        /// inherited by the child process when a new process is opened, e.g. in
        /// [`crate::process::execute_process`].
        close_on_exec => O_CLOEXEC;

        /// If this flag is set, when [`Self::open`] is called, the existing file contents will be
        /// truncated to length 0.
        ///
        /// If the file is set to read-only, then setting this flag will also set the file to
        /// read-write mode, as read-only + truncate is undefined behaviour.
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

        /// If this flag is set, when [`Self::open`] is called, neither the opening itself nor any
        /// subsequent I/O operations on the file shall cause the calling process to wait.
        non_blocking => O_NONBLOCK;

        /// If this flag is set, when [`Self::open`] is called, the file itself won't be opened.
        /// Only operations on the file descriptor level will do anything; all others will fail
        /// with [`Errno::Ebadf`].
        path_only => O_PATH;

        /// If this flag is set, when [`Self::open`] is called, then an unnamed temporary regular
        /// file will be created.
        ///
        /// Note that when this flag is set, the path given to [`Self::open`] must be a directory
        /// with write access, _not_ the file name.
        ///
        /// This flag _must_ be specified with either [`Self::write_only`] or [`Self::read_write`].
        /// Setting this flag with the [`Self::read_only`] option set will change option to
        /// [`Self::read_write`].
        create_temp => O_TMPFILE;

        /// If this flag is set, when [`Self::open`] is called, write operations on the file will
        /// be done synchronously.
        ///
        /// Put another way, any write operations will only return once all underlying hardware I/O
        /// operations have completed.
        sync => O_SYNC;
    );

    file_permissions_setter!(
        /// If this bit is set, when executing the file, the user ID will be set to that of the
        /// file owner.
        set_uid => S_ISUID;

        /// If this bit is set, when executing the file, the group ID will be set to that of the
        /// owning group.
        set_git => S_ISGID;

        /// If a directory has this bit set, its files can be deleted only by the file/directory
        /// owner or root.
        sticky => S_ISVTX;

        /// If this bit is set, the file owner can read the file.
        owner_read => S_IRUSR;

        /// If this bit is set, the file owner can write to the file.
        owner_write => S_IWUSR;

        /// If this bit is set, the file owner can execute the file.
        owner_execute => S_IXUSR;

        /// If this bit is set, the owning group can read the file.
        group_read => S_IRGRP;

        /// If this bit is set, the owning group can write to the file.
        group_write => S_IWGRP;

        /// If this bit is set, the owning group can execute the file.
        group_execute => S_IXGRP;

        /// If this bit is set, other users can read the file.
        other_read => S_IROTH;

        /// If this bit is set, other users can write to the file.
        other_write => S_IWOTH;

        /// If this bit is set, other users can execute the file.
        other_execute => S_IXOTH;
    );

    /// Ensures that any invalid flag combos are remedied.
    fn make_flags_valid(&mut self, last_changed: OpenFlags, value: bool) {
        match (last_changed, value) {
            (OpenFlags::O_RDONLY, true) => {
                // O_TRUNC and O_RDONLY is UB
                self.open_flags.remove(OpenFlags::O_TRUNC);
                // Can't create a tempfile in read-only mode
                self.open_flags.remove(OpenFlags::O_TMPFILE);
            }
            (OpenFlags::O_CREAT, false) => {
                // O_EXCL without O_CREAT is UB
                self.open_flags.remove(OpenFlags::O_EXCL);
            }
            (OpenFlags::O_EXCL, true) => {
                // O_EXCL without O_CREAT is UB
                self.open_flags.insert(OpenFlags::O_CREAT);
            }
            (OpenFlags::O_TRUNC, true) => {
                if !self
                    .open_flags
                    .intersects(OpenFlags::O_WRONLY | OpenFlags::O_RDWR)
                {
                    // O_TRUNC and O_RDONLY is UB
                    self.open_flags.insert(OpenFlags::O_RDWR);
                }
            }
            (OpenFlags::O_TMPFILE, true) => {
                if !self
                    .open_flags
                    .intersects(OpenFlags::O_WRONLY | OpenFlags::O_RDWR)
                {
                    // Can't create a tempfile in read-only mode
                    self.open_flags.insert(OpenFlags::O_RDWR);
                }
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
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::assert_err;

    const THIS_PATH: &str = "src/fs/open_options.rs";

    #[test_case]
    fn oo_new() {
        let oo = OpenOptions::new();
        assert_eq!(oo.open_flags, OpenFlags::default());
    }

    #[test_case]
    fn oo_ro() {
        let mut oo = OpenOptions::new();
        oo.close_on_exec(false);
        oo.write_only().read_only();
        assert_eq!(oo.open_flags, OpenFlags::O_RDONLY);
    }

    #[test_case]
    fn oo_wo() {
        let mut oo = OpenOptions::new();
        oo.close_on_exec(false);
        oo.write_only();
        assert_eq!(oo.open_flags, OpenFlags::O_WRONLY);
    }

    #[test_case]
    fn oo_wr() {
        let mut oo = OpenOptions::new();
        oo.close_on_exec(false);
        oo.write_only().read_only().read_write();
        assert_eq!(oo.open_flags, OpenFlags::O_RDWR);
    }

    #[test_case]
    fn no_excl_without_creat() {
        let mut oo = OpenOptions::new();
        oo.close_on_exec(false);

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

    #[test_case]
    fn no_trunc_and_ro() {
        let mut oo = OpenOptions::new();
        assert_eq!(oo.open_flags, OpenFlags::default());
        oo.close_on_exec(false);

        oo.truncate(true);
        assert_eq!(oo.open_flags, OpenFlags::O_RDWR | OpenFlags::O_TRUNC);

        oo.read_only();
        assert_eq!(oo.open_flags, OpenFlags::O_RDONLY);

        oo.write_only();
        assert_eq!(oo.open_flags, OpenFlags::O_WRONLY);

        oo.truncate(true);
        assert_eq!(oo.open_flags, OpenFlags::O_WRONLY | OpenFlags::O_TRUNC);

        oo.truncate(false);
        assert_eq!(oo.open_flags, OpenFlags::O_WRONLY);

        oo.truncate(true);
        assert_eq!(oo.open_flags, OpenFlags::O_WRONLY | OpenFlags::O_TRUNC);

        oo.read_only();
        assert_eq!(oo.open_flags, OpenFlags::O_RDONLY);
    }

    #[test_case]
    fn open_ro() {
        let _ = OpenOptions::new().open(THIS_PATH).unwrap();
    }

    #[test_case]
    fn open_dne() {
        assert_err!(
            OpenOptions::new().open("/akhflskdjnjcnds/sgsg/zsgsgsg"),
            Errno::Enoent
        );
    }

    #[test_case]
    fn no_tmp_and_ro() {
        let mut oo = OpenOptions::new();
        assert_eq!(oo.open_flags, OpenFlags::default());
        oo.close_on_exec(false);

        oo.create_temp(true);
        assert_eq!(oo.open_flags, OpenFlags::O_RDWR | OpenFlags::O_TMPFILE);

        oo.read_only();
        assert_eq!(oo.open_flags, OpenFlags::O_RDONLY);

        oo.write_only();
        assert_eq!(oo.open_flags, OpenFlags::O_WRONLY);

        oo.create_temp(true);
        assert_eq!(oo.open_flags, OpenFlags::O_WRONLY | OpenFlags::O_TMPFILE);

        oo.read_only();
        assert_eq!(oo.open_flags, OpenFlags::O_RDONLY);
    }

    #[test_case]
    fn set_mode_fp() {
        let mut oo = OpenOptions::new();
        assert_eq!(oo.file_permissions, FilePermissions::default());

        oo.set_mode(FilePermissions::all());
        assert_eq!(oo.file_permissions, FilePermissions::all());

        oo.set_mode(FilePermissions::empty());
        assert_eq!(oo.file_permissions, FilePermissions::empty());
    }

    #[test_case]
    fn set_mode_usize() {
        let mut oo = OpenOptions::new();
        assert_eq!(oo.file_permissions, FilePermissions::default());

        oo.set_mode(0o700);
        assert_eq!(
            oo.file_permissions,
            FilePermissions::S_IRUSR | FilePermissions::S_IWUSR | FilePermissions::S_IXUSR
        );

        oo.set_mode(0o007);
        assert_eq!(
            oo.file_permissions,
            FilePermissions::S_IROTH | FilePermissions::S_IWOTH | FilePermissions::S_IXOTH
        );
    }

    #[test_case]
    fn set_mode_mask() {
        let mut oo = OpenOptions::new();
        assert_eq!(oo.file_permissions, FilePermissions::default());

        oo.set_mode(0xffff_ffff_ffff_ffff);
        assert_eq!(oo.file_permissions, FilePermissions::all());
    }
}
