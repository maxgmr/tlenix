use crate::{
    fs::{FileDescriptor, FileStatsRaw},
    ipc::SigInfoRaw,
    process::ExitStatus,
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
impl From<ExitStatus> for SyscallArg {
    fn from(value: ExitStatus) -> Self {
        // We only care about the raw bytes; we don't care how Rust interprets them.
        #[allow(clippy::cast_sign_loss)]
        Self(i32::from(value) as isize as usize)
    }
}
// Macro to implement From<T> for SyscallArg where the only thing needed is `Self(T as isize as
// usize)`.
macro_rules! impl_from_syscallarg_for_as_isize {
    [$($t:ty),+] => {
        $(impl From<$t> for SyscallArg {
           fn from(value: $t) -> Self {
               // OK to lose sign here. We're simply reinterpreting bytes, the underlying syscall
               // doesn't care about the Rust data type.
               #[allow(clippy::cast_sign_loss)]
               // OK to allow this; we only truncate on targets with 32-bit pointers. This is an
               // x86_64-only program.
               #[allow(clippy::cast_possible_truncation)]
               Self(value as isize as usize)
           }
       })+
    };
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
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    *const u8,
    *const *const u8,
    *mut u8,
    *mut FileStatsRaw,
    *mut SigInfoRaw,
    *const usize,
    *mut usize
];
impl_from_syscallarg_for_as_isize![i8, i16, i32, i64, i128, isize];
