//! Minimal Tlenix program. Can be used as a blueprint for others.

#![warn(
    missing_docs,
    missing_debug_implementations,
    rust_2018_idioms,
    clippy::all,
    clippy::pedantic
)]
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![cfg_attr(test, test_runner(tlenix_core::custom_test_runner))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]

extern crate alloc;

use alloc::string::{String, ToString};
use core::panic::PanicInfo;

use getargs::{Arg, Options};
use tlenix_core::{
    EnvVar, Errno, eprintln, parse_argv_envp, println,
    process::{self, ExitStatus},
    try_exit,
};

const PANIC_TITLE: &str = "hello";

const EMBARRASSED_MSG: &str = "Hey... That's me! >_<";

core::arch::global_asm! {
    ".global _start",
    "_start:",
    "mov rdi, rsp",
    "call start"
}

/// Minimal Tlenix program. Says hello.
///
/// Intended to be used as a blueprint/reference for other Tlenix programs.
///
/// # Safety
///
/// This program must be passed appropriate `execve`-compatible args.
#[unsafe(no_mangle)]
#[allow(unused_variables)]
unsafe extern "C" fn start(stack_top: *const usize) -> ! {
    #[cfg(test)]
    {
        test_main();
        process::exit(ExitStatus::ExitSuccess);
    }

    // HACK: This stops the compiler from complaining when building the test/debug target
    #[allow(unreachable_code)]
    #[allow(clippy::no_effect)]
    ();

    // SAFETY: This function is being called right at the start of execution before anything else.
    // The stack pointer is retrieved directly from the function args.
    let (argv, envp) = match unsafe { parse_argv_envp(stack_top) } {
        Ok(argv_envp) => argv_envp,
        Err(errno) => process::exit(ExitStatus::ExitFailure(errno as i32)),
    };

    let exit_code = main(&argv, &envp);

    process::exit(exit_code);
}

fn main(args: &[String], _env_vars: &[EnvVar]) -> ExitStatus {
    match try_exit!(get_name(args)) {
        Some(name) if name.as_str() == "Tlenix" => println!("{EMBARRASSED_MSG}"),
        Some(name) => println!("Hello, {name}!"),
        None => println!("Hello!"),
    }

    ExitStatus::ExitSuccess
}

fn get_name(args: &[String]) -> Result<Option<String>, Errno> {
    let mut opts = Options::new(args.iter().map(String::as_str).skip(1));
    while let Some(arg) = opts.next_arg().map_err(|_| Errno::Einval)? {
        match arg {
            Arg::Short('n') | Arg::Long("name") => {
                return Ok(Some(opts.value().map_err(|_| Errno::Einval)?.to_string()));
            }
            _ => {}
        }
    }
    Ok(None)
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    eprintln!("{PANIC_TITLE} {info}");
    process::exit(ExitStatus::ExitFailure(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn no_arg() {
        assert_eq!(
            get_name(&[
                "hello".to_string(),
                "-ds".to_string(),
                "-e".to_string(),
                "-N".to_string(),
                "--blahblahblah".to_string()
            ]),
            Ok(None)
        );
    }

    #[test_case]
    fn bad_name_arg() {
        assert_eq!(
            get_name(&["hello".to_string(), "-n".to_string()]),
            Err(Errno::Einval)
        );
    }

    #[test_case]
    fn name_arg_short() {
        assert_eq!(
            get_name(&["hello".to_string(), "-n".to_string(), "Max".to_string()]),
            Ok(Some("Max".to_string()))
        );
    }

    #[test_case]
    fn name_arg_no_space() {
        assert_eq!(
            get_name(&["hello".to_string(), "-nMichayla".to_string()]),
            Ok(Some("Michayla".to_string()))
        );
    }

    #[test_case]
    fn name_arg_long() {
        assert_eq!(
            get_name(&[
                "hello".to_string(),
                "--name".to_string(),
                "马克斯".to_string()
            ]),
            Ok(Some("马克斯".to_string()))
        );
    }
}
