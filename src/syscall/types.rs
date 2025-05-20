use crate::{
    ExitStatus, NixBytes,
    fs::{FileDescriptor, FileStatRaw},
    nix_str::NixString,
};

/// A syscall argument. A newtype wrapper around the [`core::usize`] type.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SyscallArg(usize);
impl From<SyscallArg> for usize {
    fn from(value: SyscallArg) -> Self {
        value.0
    }
}
impl From<NixString> for SyscallArg {
    fn from(value: NixString) -> Self {
        Self(value.as_ptr() as usize)
    }
}
impl From<NixBytes> for SyscallArg {
    fn from(value: NixBytes) -> Self {
        Self(value.as_ptr() as usize)
    }
}
impl From<FileDescriptor> for SyscallArg {
    fn from(value: FileDescriptor) -> Self {
        Self(value.into())
    }
}
// Macro to implement From<T> for SyscallArg where the only thing needed is `Self(T as usize)`.
macro_rules! impl_from_syscallarg_for_as_usize {
    [$($t:ty),+] => {
        $(impl From<$t> for SyscallArg {
           fn from(value: $t) -> Self {
               Self(value as usize)
           }
       })+
    };
}
impl_from_syscallarg_for_as_usize![
    usize,
    ExitStatus,
    *const u8,
    *const *const u8,
    *mut u8,
    *mut FileStatRaw
];
