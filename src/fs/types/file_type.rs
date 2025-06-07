//! The [`FileType`] type.

use crate::Errno;

/// Bit mask for the file type bit field.
const S_IFMT: u32 = 0o0_170_000;

/// The type of a given [`crate::fs::File`].
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FileType {
    /// A [Unix domain socket](https://en.wikipedia.org/wiki/Unix_domain_socket).
    Socket = 0o0_140_000,
    /// A [symbolic link](https://en.wikipedia.org/wiki/Symbolic_link).
    SymbolicLink = 0o0_120_000,
    /// A regular file.
    RegularFile = 0o0_100_000,
    /// A [block device file](https://en.wikipedia.org/wiki/Device_file#Block_devices).
    BlockDevice = 0o0_060_000,
    /// A file directory.
    Directory = 0o0_040_000,
    /// A [character device file](https://en.wikipedia.org/wiki/Device_file#Character_devices).
    CharacterDevice = 0o0_020_000,
    /// A first-in-first-out [named pipe](https://en.wikipedia.org/wiki/Named_pipe).
    Fifo = 0o0_010_000,
}
impl TryFrom<u32> for FileType {
    type Error = Errno;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let masked_value = value & S_IFMT;

        if masked_value == (Self::Socket as u32) {
            Ok(Self::Socket)
        } else if masked_value == (Self::SymbolicLink as u32) {
            Ok(Self::SymbolicLink)
        } else if masked_value == (Self::RegularFile as u32) {
            Ok(Self::RegularFile)
        } else if masked_value == (Self::BlockDevice as u32) {
            Ok(Self::BlockDevice)
        } else if masked_value == (Self::Directory as u32) {
            Ok(Self::Directory)
        } else if masked_value == (Self::CharacterDevice as u32) {
            Ok(Self::CharacterDevice)
        } else if masked_value == (Self::Fifo as u32) {
            Ok(Self::Fifo)
        } else {
            Err(Errno::Einval)
        }
    }
}
