use crate::ExitStatus;

/// A syscall argument. A newtype wrapper around the [`core::usize`] type.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SyscallArg(usize);
impl From<SyscallArg> for usize {
    fn from(value: SyscallArg) -> Self {
        value.0
    }
}
impl From<usize> for SyscallArg {
    fn from(value: usize) -> Self {
        Self(value)
    }
}
impl From<ExitStatus> for SyscallArg {
    fn from(value: ExitStatus) -> Self {
        Self(value as usize)
    }
}
