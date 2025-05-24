use crate::{
    ExitStatus,
    fs::{DirEntRaw, FileDescriptor, FileStatRaw},
};

/// A syscall argument. A newtype wrapper around the [`core::usize`] type.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SyscallArg(usize);
impl From<SyscallArg> for usize {
    fn from(value: SyscallArg) -> Self {
        value.0
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
               // OK to lose sign here. We're simply reinterpreting bytes, the underlying syscall
               // doesn't care about the Rust data type.
               #[allow(clippy::cast_sign_loss)]
               // OK to allow this; we only truncate on targets with 32-bit pointers. This is an
               // x86_64-only program.
               #[allow(clippy::cast_possible_truncation)]
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
    *mut FileStatRaw,
    *mut DirEntRaw,
    i32,
    i64,
    u64,
    *const usize
];
