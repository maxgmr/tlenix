//! `mash` = **Ma**x's **Sh**ell. Tlenix's shell program! Provides a command-line user interface
//! for Tlenix.

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

extern crate alloc;

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use core::panic::PanicInfo;
use num_enum::TryFromPrimitive;

use tlenix_core::{
    Console, EnvVar, Errno, align_stack_pointer, eprintln,
    fs::{self},
    print,
    process::{self, ExitStatus},
    system,
};

const MASH_PANIC_TITLE: &str = "mash";

const PROMPT_START: &str = "\u{001b}[94mmash\u{001b}[0m";
const PROMPT_FINISH: &str = "\u{001b}[92;1m:}\u{001b}[0m";

/// Used as a backup just in case the current working directory can't be determined.
const CWD_NAME_BACKUP: &str = "?";

/// Maximum line size.
const LINE_MAX: usize = 1 << 12;

/// Lines starting with this character in the environment variable file are ignored.
const ENVIRONMENT_COMMENT: char = '#';

/// Name of the `PATH` environment variable.
const PATH_ENV_VAR_NAME: &str = "PATH";

/// Character separating the various `PATH` environment variable paths.
const PATH_SEPARATOR: char = ':';

// Home directory.
#[cfg(debug_assertions)]
const HOME_DIR: &str = "/";
#[cfg(not(debug_assertions))]
const HOME_DIR: &str = "/root";

// Location where environment variables are stored.
#[cfg(debug_assertions)]
const ENV_VAR_PATH: &str = "os_files/etc/environment";
#[cfg(not(debug_assertions))]
const ENV_VAR_PATH: &str = "/etc/environment";

/// Entry point.
///
/// # Panics
///
/// This function panics if the system fails to power off properly.
#[unsafe(no_mangle)]
extern "C" fn _start() -> ! {
    align_stack_pointer!();

    #[cfg(test)]
    process::exit(process::ExitStatus::ExitSuccess);

    // HACK: This stops the compiler from complaining when building the test/debug target
    #[allow(unreachable_code)]
    #[allow(clippy::no_effect)]
    ();

    let console = Console::open().unwrap();
    loop {
        print_prompt();

        // Get argv.
        let line = console.read_line(LINE_MAX).unwrap();
        let line_string = String::from_utf8(line).unwrap();
        let mut argv: Vec<&str> = line_string.split_whitespace().collect();

        // Read env vars.
        let env_vars = read_env_vars();
        let envp = env_vars.iter().map(String::from).collect::<Vec<String>>();

        // Do nothing if nothing was typed
        if argv.is_empty() {
            eprintln!("doing nothin'");
            continue;
        }

        match (argv[0], argv.len()) {
            ("exit", 1) => process::exit(process::ExitStatus::ExitSuccess),
            ("poweroff", 1) => {
                let errno = system::power_off().unwrap_err();
                eprintln!("poweroff fail: {}", errno.as_str());
            }
            ("reboot", 1) => {
                let errno = system::reboot().unwrap_err();
                eprintln!("reboot fail: {}", errno.as_str());
            }
            ("cd", 1) => {
                if let Err(e) = fs::change_dir(HOME_DIR) {
                    eprintln!("{e}");
                }
            }
            ("cd", 2) => {
                if let Err(e) = fs::change_dir(argv[1]) {
                    eprintln!("{e}");
                }
            }
            (_, _) => {
                let new_argv0 = match program_path_subst(argv[0], &env_vars) {
                    Ok(new_argv0) => new_argv0,
                    Err(Errno::Enoent) => {
                        eprintln!("Unrecognised command.");
                        continue;
                    }
                    Err(errno) => {
                        eprintln!("Program path substitute fail: {errno}");
                        continue;
                    }
                };
                argv[0] = &new_argv0;

                match process::execute_process(&argv, &envp) {
                    Ok(ExitStatus::ExitFailure(code)) => {
                        if let Ok(errno) = Errno::try_from_primitive(code) {
                            eprintln!("{}: {}", argv[0], errno);
                        } else {
                            eprintln!("{}: Process exited with failure code {}.", argv[0], code);
                        }
                    }
                    Ok(ExitStatus::Terminated(signo)) => {
                        eprintln!("{}: Process terminated {}", argv[0], signo);
                    }
                    Err(e) => {
                        eprintln!("{}: {}", argv[0], e);
                    }
                    #[allow(unused_variables)]
                    other => {
                        #[cfg(debug_assertions)]
                        eprintln!("{}: {:?}", argv[0], other);
                    }
                }
            }
        }
    }
}

/// Read and parse the environment files from the disk.
///
/// If things go wrong, this function will print a warning and return an empty vec.
fn read_env_vars() -> Vec<EnvVar> {
    let ev_file = match fs::OpenOptions::new().open(ENV_VAR_PATH) {
        Ok(file) => file,
        Err(e) => {
            return env_var_read_fail("failed to open", e);
        }
    };
    let ev_file_string = match ev_file.read_to_string() {
        Ok(ev_file_string) => ev_file_string,
        Err(e) => {
            return env_var_read_fail("failed to read", e);
        }
    };

    let mut env_vars = Vec::new();
    let mut line_num = 0;
    for line in ev_file_string.split('\n') {
        line_num += 1;

        if !line.starts_with(ENVIRONMENT_COMMENT) && !line.trim().is_empty() {
            match EnvVar::try_from(line) {
                Ok(env_var) => env_vars.push(env_var),
                Err(e) => {
                    eprintln!("Error parsing line {line_num}: {line}");
                    return env_var_read_fail("invalid format in", e);
                }
            }
        }
    }
    env_vars
}

fn env_var_read_fail(reason: &'static str, e: Errno) -> Vec<EnvVar> {
    eprintln!(
        "Warning: {reason} `{ENV_VAR_PATH}`. Environment variables will be unavailable this session. ({e})"
    );
    Vec::new()
}

/// Print the MASH shell prompt.
fn print_prompt() {
    let cwd_backup = String::from(CWD_NAME_BACKUP);
    let cwd = fs::get_cwd().unwrap_or(cwd_backup);
    let basename =
        &cwd.rsplit_once('/').map_or(
            cwd.as_str(),
            |(_, last)| if last.is_empty() { "/" } else { last },
        );

    print!("{PROMPT_START} {basename} {PROMPT_FINISH} ");
}

/// Parse the first argv entry as a program.
///
/// # Errors
///
/// This function returns [`Errno::Einval`] if `env_vars` does not contain a `PATH` env var.
///
/// This function returns [`Errno::Enoent`] if the program does not exist.
fn program_path_subst(argv0: &str, env_vars: &[EnvVar]) -> Result<String, Errno> {
    if argv0.contains('/') {
        // Is already a file path. Ignore PATH.
        return Ok(argv0.to_string());
    }

    // Get the path variable from env vars.
    let path_env_var = env_vars
        .iter()
        .find(|ev| ev.key == PATH_ENV_VAR_NAME)
        .ok_or(Errno::Einval)?
        .value
        .as_str();

    // Test all the different paths in PATH.
    for path in path_env_var.split(PATH_SEPARATOR) {
        // Append the argument onto the current path prefix.
        let mut candidate_path = String::with_capacity(path.len() + argv0.len() + 1);
        candidate_path.push_str(path);
        if !candidate_path.ends_with('/') {
            candidate_path.push('/');
        }
        candidate_path.push_str(argv0);

        // See if you're able to access the assembled path.
        let Ok(file) = fs::OpenOptions::new()
            .path_only(true)
            .open(candidate_path.as_str())
        else {
            // Candidate doesn't exist (most likely) or there was another error. Move on to the
            // next candidate.
            continue;
        };

        let Ok(stats) = file.stat() else {
            continue;
        };
        // If the file isn't a regular file, try a different option.
        if stats.file_type != fs::FileType::RegularFile {
            continue;
        }

        let fp = stats.file_permissions;
        if !fp.intersects(
            fs::FilePermissions::S_IXUSR
                | fs::FilePermissions::S_IXGRP
                | fs::FilePermissions::S_IXOTH,
        ) {
            // File is not executable.
            continue;
        }

        // The file exists, is a regular file, and is executable. We've got one. Return it.
        return Ok(candidate_path);
    }
    // No candidate paths matched. Unknown command.
    Err(Errno::Enoent)
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    tlenix_core::eprintln!("{} {}", MASH_PANIC_TITLE, info);
    process::exit(process::ExitStatus::ExitFailure(1))
}
