//! Prints all the environment variables.

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

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use core::panic::PanicInfo;

use getargs::{Arg, Options};
use tlenix_core::{
    EnvVar, Errno, eprintln, parse_argv_envp, println,
    process::{self, ExitStatus},
    try_exit,
};

const PANIC_TITLE: &str = "printenv";

const PRINTENV_SEPARATOR: &str = "\n";

core::arch::global_asm! {
    ".global _start",
    "_start:",
    "mov rdi, rsp",
    "call start"
}

/// Prints all the environment variables.
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

fn main(args: &[String], env_vars: &[EnvVar]) -> ExitStatus {
    let filter = try_exit!(get_filter(args));
    let filtered_env_vars = filter_env_vars(env_vars, &filter);
    println!("{}", format_string(&filtered_env_vars, filter.is_empty()));
    ExitStatus::ExitSuccess
}

fn get_filter(args: &[String]) -> Result<Vec<&str>, Errno> {
    let mut opts = Options::new(args.iter().map(String::as_str).skip(1));
    let mut filter = Vec::with_capacity(args.len());
    while let Some(arg) = opts.next_arg().map_err(|_| Errno::Einval)? {
        if let Arg::Positional(val) = arg {
            filter.push(val);
        }
    }
    Ok(filter)
}

fn filter_env_vars<'a>(env_vars: &'a [EnvVar], filter: &[&str]) -> Vec<&'a EnvVar> {
    if filter.is_empty() {
        env_vars.iter().collect()
    } else {
        env_vars
            .iter()
            .filter(|ev| filter.contains(&ev.key.as_str()))
            .collect()
    }
}

fn format_string(env_vars: &[&EnvVar], include_keys: bool) -> String {
    env_vars
        .iter()
        .map(|ev| {
            if include_keys {
                ev.to_string()
            } else {
                ev.value.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join(PRINTENV_SEPARATOR)
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    eprintln!("{PANIC_TITLE} {info}");
    process::exit(ExitStatus::ExitFailure(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! get_filter_test {
        ($fn_name:ident [$($arg:expr),*] => [$($expected:expr),*]) => {
           #[test_case]
           fn $fn_name() {
               let input = ["printenv".to_string(), $($arg.to_string()),*];
               let result = get_filter(&input).unwrap();
               let expected: &[&str] = &[$($expected),*][..];
               assert_eq!(&result, expected);
           }
        };
    }
    get_filter_test!(get_filter_empty [] => []);
    get_filter_test!(get_filter_one ["PATH"] => ["PATH"]);
    get_filter_test!(get_filter_three ["TEST1", "PATH", "TEST2"] => ["TEST1", "PATH", "TEST2"]);
    get_filter_test!(get_filter_empty_flags ["--myflag", "-f", "-Alq"] => []);
    get_filter_test!(get_filter_flags_interspersed ["--myflag", "TEST1", "-Alq", "TEST2", "-z"] => ["TEST1", "TEST2"]);

    macro_rules! filter_ev_test {
        ($fn_name:ident([$(($ev_k:expr, $ev_v:expr)),*], [$($f:expr),*]) => [$(($ex_k:expr, $ex_v:expr)),*]) => {
            #[test_case]
            fn $fn_name() {
                let input: &[EnvVar] = &[$(EnvVar {key: $ev_k.to_string(), value: $ev_v.to_string()}),*];
                let filter: &[&str] = &[$($f),*][..];
                let expected_owned: &[EnvVar] = &[$(EnvVar {key: $ex_k.to_string(), value: $ex_v.to_string()}),*];
                let expected: Vec<&EnvVar> = expected_owned.iter().collect();
                assert_eq!(filter_env_vars(input, filter), expected);
            }
        };
    }
    filter_ev_test!(empty_filter([("K1", "v1"), ("K2", "v2")], []) => [("K1", "v1"), ("K2", "v2")]);
    filter_ev_test!(filter_one([("K1", "v1"), ("K2", "v2")], ["K2"]) => [("K2", "v2")]);
    filter_ev_test!(filter_dne([("K1", "v1"), ("K2", "v2")], ["NOT_A_KEY"]) => []);
    filter_ev_test!(filter_multiple([("K1", ""), ("K2", "abc"), ("K3", "123")], ["K3", "K1", "NOT_A_KEY"]) => [("K1", ""), ("K3", "123")]);

    #[test_case]
    fn format_string_no_keys() {
        let evs_owned = [
            EnvVar {
                key: "K1".to_string(),
                value: "123".to_string(),
            },
            EnvVar {
                key: "K2".to_string(),
                value: "abc".to_string(),
            },
        ];
        let evs: Vec<&EnvVar> = evs_owned.iter().collect();
        assert_eq!("123\nabc", &format_string(&evs, false));
    }

    #[test_case]
    fn format_string_with_keys() {
        let evs_owned = [
            EnvVar {
                key: "K1".to_string(),
                value: "123".to_string(),
            },
            EnvVar {
                key: "K2".to_string(),
                value: "abc".to_string(),
            },
        ];
        let evs: Vec<&EnvVar> = evs_owned.iter().collect();
        assert_eq!("K1=123\nK2=abc", &format_string(&evs, true));
    }

    #[test_case]
    fn format_string_empty_value() {
        let evs_owned = [EnvVar {
            key: "K1".to_string(),
            value: String::new(),
        }];
        let evs: Vec<&EnvVar> = evs_owned.iter().collect();
        assert_eq!("", &format_string(&evs, false));
        assert_eq!("K1=", &format_string(&evs, true));
    }

    #[test_case]
    fn format_string_empty() {
        let evs: Vec<&EnvVar> = Vec::new();
        assert_eq!("", &format_string(&evs, false));
        assert_eq!("", &format_string(&evs, true));
    }
}
