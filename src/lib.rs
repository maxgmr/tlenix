//! Library crate for the [tlenix](https://github.com/maxgmr/tlenix) `x86_64` operating system.
#![cfg(all(target_os = "linux", target_arch = "x86_64"))]
#![warn(
    missing_docs,
    missing_debug_implementations,
    rust_2018_idioms,
    clippy::all,
    clippy::pedantic,
    clippy::unwrap_used,
    clippy::todo
)]
#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks, never_type)]
#![test_runner(test_framework::custom_test_runner)]
#![reexport_test_harness_main = "test_main"]

#[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
compile_error!("This crate only functions on x86_64 linux targets.");

// Make sure the compiler includes `alloc`
#[allow(unused_extern_crates)]
extern crate alloc;

mod allocator;
mod console;
pub mod fs;
mod nix_bytes;
mod nix_str;
mod print;
pub mod process;
mod syscall;
pub mod system;
mod test_framework;
pub mod thread;

#[cfg(test)]
pub(crate) mod test_utils;

// RE-EXPORTS
pub use console::Console;
pub use nix_bytes::{NixBytes, vec_into_nix_bytes};
pub use nix_str::{NixString, vec_into_nix_strings};
pub use print::{__print_err, __print_str};
pub use syscall::{Errno, SyscallArg, SyscallNum};
pub use test_framework::custom_test_runner;

/// The null byte, commonly used for terminating strings and defining null pointers.
pub(crate) const NULL_BYTE: u8 = b'\0';

/// The two constants specified by the C standard denoting the success or failure of an process.
#[repr(usize)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExitStatus {
    /// C standard success exit code.
    ExitSuccess = 0_usize,
    /// C standard failure exit code.
    ExitFailure = 1_usize,
}

/// Aligns the stack pointer. Intended for use right at the beginning of execution.
///
/// SAFETY: Valid ASM instruction with valid, statically-chosen arguments.
#[macro_export]
macro_rules! align_stack_pointer {
    // This can't be called as a function; it must be directly invoked right at the start.
    () => {
        unsafe {
            core::arch::asm!("and rsp, -16", options(nostack));
        }
    };
}

/// Entry point for library tests.
#[cfg(test)]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    align_stack_pointer!();
    test_main();
    process::exit(ExitStatus::ExitSuccess);
}

/// Panic handler for library tests.
#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    test_framework::test_panic_handler(info)
}
