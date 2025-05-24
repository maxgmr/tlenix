//! The [`FilePermissions`] bitflags.

use core::default::Default;

bitflags::bitflags! {
    /// The attributes of a given file. See
    /// [here](https://www.man7.org/linux/man-pages/man3/mode_t.3type.html) for more details.
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct FilePermissions: usize {
        /// set-user-ID: Set process effective user ID on `execve(2)`.
        const S_ISUID = 0o04_000;
        /// set-group-ID: Set process effective group ID on `execve(2)`.
        const S_ISGID = 0o02_000;
        /// Sticky bit: restricted deletion flag.
        const S_ISVTX = 0o01_000;
        /// Owner can read.
        const S_IRUSR = 0o00_400;
        /// Owner can write.
        const S_IWUSR = 0o00_200;
        /// Owner can execute/search.
        const S_IXUSR = 0o00_100;
        /// Group can read.
        const S_IRGRP = 0o00_040;
        /// Group can write.
        const S_IWGRP = 0o00_020;
        /// Group can execute/search.
        const S_IXGRP = 0o00_010;
        /// Others can read.
        const S_IROTH = 0o00_004;
        /// Others can write.
        const S_IWOTH = 0o00_002;
        /// Others can execute/search.
        const S_IXOTH = 0o00_001;
    }
}

impl Default for FilePermissions {
    fn default() -> Self {
        // Default = 0o644
        let mut result = Self::empty();
        result.insert(Self::S_IRUSR | Self::S_IWUSR | Self::S_IRGRP | Self::S_IROTH);
        result
    }
}
impl From<usize> for FilePermissions {
    fn from(value: usize) -> Self {
        Self::from_bits_truncate(value)
    }
}
