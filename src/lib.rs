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
mod args;
mod console;
pub mod fs;
pub mod ipc;
mod nix_bytes;
mod nix_str;
mod print;
pub mod process;
pub mod streams;
mod syscall;
pub mod system;
pub mod term;
mod test_framework;
pub mod thread;

#[cfg(test)]
pub(crate) mod test_utils;

// RE-EXPORTS
pub use args::{EnvVar, parse_argv_envp};
pub use console::Console;
pub use nix_bytes::NixBytes;
pub use nix_str::NixString;
pub use print::{__format, __print_err, __print_str};
pub use syscall::{Errno, SyscallArg, SyscallNum};
pub(crate) use syscall::{syscall, syscall_result};
pub use test_framework::custom_test_runner;

/// The null byte, commonly used for terminating strings and defining null pointers.
pub(crate) const NULL_BYTE: u8 = b'\0';

/// The page size of x86 Linux. (4 KiB)
pub(crate) const PAGE_SIZE: usize = 1 << 12;

/// The length limit of an individual command-line argument.
pub const ARG_LEN_LIM: usize = PAGE_SIZE;

/// The length limit of an individual environment variable.
pub const ENV_LEN_LIM: usize = PAGE_SIZE;

/// The limit on the total size of `argv` and `envp` strings.
pub const ARG_ENV_LIM: usize = PAGE_SIZE * 32;

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

/// If the given expression returns [`Ok`], unwrap it. Otherwise, return from the function with the
/// numerical error as [`process::ExitStatus::ExitFailure`].
#[macro_export]
macro_rules! try_exit {
    ($e:expr) => {
        match $e {
            Ok(val) => val,
            Err(e) => return $crate::process::ExitStatus::ExitFailure(e as i32),
        }
    };
}

/// Entry point for library tests.
#[cfg(test)]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    align_stack_pointer!();
    test_main();
    process::exit(process::ExitStatus::ExitSuccess);
}

/// Panic handler for library tests.
#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    test_framework::test_panic_handler(info)
}
