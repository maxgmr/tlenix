//! The [`LseekWhence`] type.

use crate::SyscallArg;

/// All possible values which can be sent to the `lseek` syscall to declare its functionality.
#[repr(usize)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(clippy::enum_variant_names)]
pub enum LseekWhence {
    /// The file offset is set to `offset` bytes.
    SeekSet,
    /// The file offset is set to its current location plus `offset` bytes.
    SeekCur,
    /// The file offset is set to the size of the file plus `offset` bytes.
    SeekEnd,
    /// Adjust the file offset to the next location in the file which contains data.
    SeekData,
    /// Adjust the file offset to the next hole in the file. If no holes, the offset is set to the
    /// end of the file.
    SeekHole,
}
impl From<LseekWhence> for SyscallArg {
    fn from(value: LseekWhence) -> Self {
        Self::from(value as usize)
    }
}
