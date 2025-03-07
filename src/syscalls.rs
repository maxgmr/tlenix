//! A list of Linux syscalls.
#![allow(clippy::must_use_candidate)]

use core::arch::asm;

mod errno;
mod nums;

pub use errno::Errno;
pub use nums::SyscallNum;

/// Invoke a Linux syscall, getting a [`usize`] in return.
///
/// # Safety
///
/// Syscalls are inherently unsafe- the caller must ensure safety.
#[macro_export]
macro_rules! syscall {
    ($cn:expr) => {
        $crate::syscalls::syscall_0($cn)
    };
    ($cn:expr, $a0:expr) => {
        $crate::syscalls::syscall_1($cn, $a0)
    };
    ($cn:expr, $a0:expr, $a1:expr) => {
        $crate::syscalls::syscall_2($cn, $a0, $a1)
    };
    ($cn:expr, $a0:expr, $a1:expr, $a2:expr) => {
        $crate::syscalls::syscall_3($cn, $a0, $a1, $a2)
    };
    ($cn:expr, $a0:expr, $a1:expr, $a2:expr, $a3:expr) => {
        $crate::syscalls::syscall_4($cn, $a0, $a1, $a2, $a3)
    };
    ($cn:expr, $a0:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr) => {
        $crate::syscalls::syscall_5($cn, $a0, $a1, $a2, $a3, $a4)
    };
    ($cn:expr, $a0:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr, $a5:expr) => {
        $crate::syscalls::syscall_6($cn, $a0, $a1, $a2, $a3, $a4, $a5)
    };
}

/// Invoke a Linux syscall, returning a [`Result`].
///
/// If the syscall is successful, the value is returned within the [`Ok`].
///
/// If the syscall is _unsuccessful_, an [`Errno`] is returned within the [`Ok`].
#[macro_export]
macro_rules! syscall_result {
    ($($arg:expr),*) => {
        $crate::Errno::__from_ret($crate::syscall!($($arg),*))
    }
}

/// Invoke a Linux syscall with 0 args.
///
/// # Safety
///
/// Syscalls are inherently unsafe- the caller must ensure safety.
#[inline]
pub unsafe fn syscall_0(call_num: SyscallNum) -> usize {
    let mut ret: usize;

    unsafe {
        asm!(
            "syscall",
            inlateout("rax") call_num as usize => ret,
            out("rcx") _, // clobbered
            out("r11") _, // clobbered
            options(nostack, preserves_flags)
        );
    }

    ret
}

/// Invoke a Linux syscall with 1 arg.
///
/// # Safety
///
/// Syscalls are inherently unsafe- the caller must ensure safety.
#[inline]
pub unsafe fn syscall_1(call_num: SyscallNum, arg0: usize) -> usize {
    let mut ret: usize;

    unsafe {
        asm!(
            "syscall",
            inlateout("rax") call_num as usize => ret,
            in("rdi") arg0,
            out("rcx") _, // clobbered
            out("r11") _, // clobbered
            options(nostack, preserves_flags)
        );
    }

    ret
}

/// Invoke a Linux syscall with 2 args.
///
/// # Safety
///
/// Syscalls are inherently unsafe- the caller must ensure safety.
#[inline]
pub unsafe fn syscall_2(call_num: SyscallNum, arg0: usize, arg1: usize) -> usize {
    let mut ret: usize;

    unsafe {
        asm!(
            "syscall",
            inlateout("rax") call_num as usize => ret,
            in("rdi") arg0,
            in("rsi") arg1,
            out("rcx") _, // clobbered
            out("r11") _, // clobbered
            options(nostack, preserves_flags)
        );
    }

    ret
}

/// Invoke a Linux syscall with 3 args.
///
/// # Safety
///
/// Syscalls are inherently unsafe- the caller must ensure safety.
#[inline]
pub unsafe fn syscall_3(call_num: SyscallNum, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let mut ret: usize;

    unsafe {
        asm!(
            "syscall",
            inlateout("rax") call_num as usize => ret,
            in("rdi") arg0,
            in("rsi") arg1,
            in("rdx") arg2,
            out("rcx") _, // clobbered
            out("r11") _, // clobbered
            options(nostack, preserves_flags)
        );
    }

    ret
}

/// Invoke a Linux syscall with 4 args.
///
/// # Safety
///
/// Syscalls are inherently unsafe- the caller must ensure safety.
#[inline]
pub unsafe fn syscall_4(
    call_num: SyscallNum,
    arg0: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
) -> usize {
    let mut ret: usize;

    unsafe {
        asm!(
            "syscall",
            inlateout("rax") call_num as usize => ret,
            in("rdi") arg0,
            in("rsi") arg1,
            in("rdx") arg2,
            in("r10") arg3,
            out("rcx") _, // clobbered
            out("r11") _, // clobbered
            options(nostack, preserves_flags)
        );
    }

    ret
}

/// Invoke a Linux syscall with 5 args.
///
/// # Safety
///
/// Syscalls are inherently unsafe- the caller must ensure safety.
#[inline]
pub unsafe fn syscall_5(
    call_num: SyscallNum,
    arg0: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
) -> usize {
    let mut ret: usize;

    unsafe {
        asm!(
            "syscall",
            inlateout("rax") call_num as usize => ret,
            in("rdi") arg0,
            in("rsi") arg1,
            in("rdx") arg2,
            in("r10") arg3,
            in("r8") arg4,
            out("rcx") _, // clobbered
            out("r11") _, // clobbered
            options(nostack, preserves_flags)
        );
    }

    ret
}

/// Invoke a Linux syscall with 6 args.
///
/// # Safety
///
/// Syscalls are inherently unsafe- the caller must ensure safety.
#[inline]
pub unsafe fn syscall_6(
    call_num: SyscallNum,
    arg0: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
) -> usize {
    let mut ret: usize;

    unsafe {
        asm!(
            "syscall",
            inlateout("rax") call_num as usize => ret,
            in("rdi") arg0,
            in("rsi") arg1,
            in("rdx") arg2,
            in("r10") arg3,
            in("r8") arg4,
            in("r9") arg5,
            out("rcx") _, // clobbered
            out("r11") _, // clobbered
            options(nostack, preserves_flags)
        );
    }

    ret
}
