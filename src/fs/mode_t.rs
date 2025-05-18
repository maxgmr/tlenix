//! The [`ModeT`] bitflags.

bitflags::bitflags! {
    /// The attributes of a given file. See
    /// [here](https://www.man7.org/linux/man-pages/man3/mode_t.3type.html) for more details.
    pub struct ModeT: usize {
        const S_ISUID = 0o04_000;
        const S_ISGID = 0o02_000;
        const S_ISVTX = 0o01_000;
        const S_IRUSR = 0o00_400;
        const S_IWUSR = 0o00_200;
        const S_IXUSR = 0o00_100;
        const S_IRGRP = 0o00_040;
        const S_IWGRP = 0o00_020;
        const S_IXGRP = 0o00_010;
        const S_IROTH = 0o00_004;
        const S_IWOTH = 0o00_002;
        const S_IXOTH = 0o00_001;
    }
}
