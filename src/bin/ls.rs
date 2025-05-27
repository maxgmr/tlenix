//! Lists the contents of the given directory.

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

use alloc::{string::String, vec::Vec};
use core::panic::PanicInfo;

use getargs::{Arg, Options};

use tlenix_core::{
    EnvVar, Errno, eprintln, fs, parse_argv_envp, println,
    process::{self, ExitStatus},
    try_exit,
};

const PANIC_TITLE: &str = "ls";

const ENTRY_SEPARATOR: &str = "\t";
const LIST_ENTRY_SEPARATOR: &str = "\n";

const THIS_DIR: &str = ".";
const SUPER_DIR: &str = "..";

const DEFAULT_PATH: &str = THIS_DIR;

const HIDDEN_PREFIX: char = '.';

core::arch::global_asm! {
    ".global _start",
    "_start:",
    "mov rdi, rsp",
    "call start"
}

/// All the things that modify `ls`'s behaviour.
#[derive(Clone, Debug, PartialEq, Eq)]
struct LsSettings<'a> {
    /// The path to the queried directory.
    path: &'a str,
    /// The text which separates the directory entries.
    separator: &'static str,
    /// Whether or not to filter out hidden files.
    filter_hidden: bool,
    /// Whether or not to filter out "." and "..".
    filter_implied: bool,
}
impl<'a> TryFrom<&'a [String]> for LsSettings<'a> {
    type Error = Errno;

    fn try_from(value: &'a [String]) -> Result<Self, Self::Error> {
        let mut opts = Options::new(value.iter().map(String::as_str).skip(1));

        let mut separator = ENTRY_SEPARATOR;
        let mut path = DEFAULT_PATH;
        let mut got_path = false;
        let mut filter_dotfiles = true;
        let mut filter_implied = true;

        while let Some(arg) = opts.next_arg().map_err(|_| Errno::Einval)? {
            match arg {
                Arg::Short('l') | Arg::Long("list" | "long") => separator = LIST_ENTRY_SEPARATOR,
                Arg::Short('a') | Arg::Long("all") => {
                    filter_dotfiles = false;
                    filter_implied = false;
                }
                Arg::Short('A') | Arg::Long("almost-all") => {
                    filter_dotfiles = false;
                    filter_implied = true;
                }
                Arg::Positional(val) if !got_path => {
                    path = val;
                    got_path = true;
                }
                _ => {}
            }
        }

        Ok(Self {
            path,
            separator,
            filter_hidden: filter_dotfiles,
            filter_implied,
        })
    }
}

/// Lists the contents of the given directory.
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
    let ls_settings = try_exit!(LsSettings::try_from(args));
    let dent_names = try_exit!(dent_names(ls_settings.path));
    let out_str = fmt_str(
        dent_names,
        ls_settings.separator,
        ls_settings.filter_hidden,
        ls_settings.filter_implied,
    );

    println!("{out_str}");

    ExitStatus::ExitSuccess
}

/// Reads the list of names from the entries of the directory at the given path.
///
/// # Errors
///
/// This function returns [`Errno`] if [`fs::OpenOptions::open`] or [`fs::File::dir_ents`] fail.
fn dent_names(path: &str) -> Result<Vec<String>, Errno> {
    Ok(fs::OpenOptions::new()
        .directory(true)
        .open(path)?
        .dir_ents()?
        .into_iter()
        .map(|d| d.name)
        .collect())
}

/// Sorts the given list of names, filters hidden files, and joins them with the given separator.
fn fmt_str(
    mut names: Vec<String>,
    separator: &str,
    filter_hidden: bool,
    filter_implied: bool,
) -> String {
    names.sort_unstable();
    names.retain(|n| {
        !(filter_hidden && n.starts_with(HIDDEN_PREFIX))
            && !(filter_implied && (n == THIS_DIR || n == SUPER_DIR))
    });
    names.join(separator)
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    eprintln!("{PANIC_TITLE} {info}");
    process::exit(ExitStatus::ExitFailure(1))
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use tlenix_core::fs;

    use super::*;

    #[test_case]
    fn fmt_str_empty() {
        let names = Vec::from(["a".to_string(), "b".to_string(), "c".to_string()]);
        let expected = "abc".to_string();
        assert_eq!(fmt_str(names, "", false, false), expected);
    }

    #[test_case]
    fn fmt_str_tab() {
        let names = Vec::from(["a".to_string(), "b".to_string(), "c".to_string()]);
        let expected = "a\tb\tc".to_string();
        assert_eq!(fmt_str(names, "\t", false, false), expected);
    }

    #[test_case]
    fn fmt_empty_str() {
        let names = Vec::new();
        let expected = String::new();
        assert_eq!(fmt_str(names, "akjshlkjehg", false, false), expected);
    }

    #[test_case]
    fn fmt_str_unsorted() {
        let names = Vec::from([
            "c".to_string(),
            "a".to_string(),
            "..".to_string(),
            "b".to_string(),
            ".".to_string(),
        ]);
        let expected = ". .. a b c";
        assert_eq!(fmt_str(names, " ", false, false), expected);
    }

    #[test_case]
    fn fmt_str_filter_hidden() {
        let names = Vec::from([
            ".c1234".to_string(),
            "b".to_string(),
            ".".to_string(),
            ".a".to_string(),
            "a".to_string(),
            "..".to_string(),
        ]);
        let expected = "a\nb";
        assert_eq!(fmt_str(names, "\n", true, false), expected);
    }

    #[test_case]
    fn fmt_str_filter_implied() {
        let names = Vec::from([
            ".b".to_string(),
            ".".to_string(),
            ".a".to_string(),
            "..".to_string(),
        ]);
        let expected = ".a\n.b";
        assert_eq!(fmt_str(names, "\n", false, true), expected);
    }

    macro_rules! lss_test {
        ($test_name:ident([$($s:literal),*] => ($path:expr, $sep:expr, $fh:expr, $fi:expr))) => {
            #[test_case]
            fn $test_name() {
                let strings = ["ls".to_string(), $($s.to_string()),*];
                let lss = LsSettings::try_from(&strings[..]).unwrap();
                let expected = LsSettings {
                    path: $path,
                    separator: $sep,
                    filter_hidden: $fh,
                    filter_implied: $fi,
                };
                assert_eq!(lss, expected);
            }
        };
    }

    lss_test!(lss_empty([] => (DEFAULT_PATH, ENTRY_SEPARATOR, true, true)));
    lss_test!(lss_dir(["/"] => ("/", ENTRY_SEPARATOR, true, true)));
    lss_test!(lss_l(["-l"] => (DEFAULT_PATH, LIST_ENTRY_SEPARATOR, true, true)));
    lss_test!(lss_l_before_dir(["-l", "mydir"] => ("mydir", LIST_ENTRY_SEPARATOR, true, true)));
    lss_test!(lss_l_after_dir(["mydir", "-l"] => ("mydir", LIST_ENTRY_SEPARATOR, true, true)));
    lss_test!(lss_extra_flags(["-bks", "mydir", "-lhk"] => ("mydir", LIST_ENTRY_SEPARATOR, true, true)));
    lss_test!(lss_long_l_after(["mydir", "--long"] => ("mydir", LIST_ENTRY_SEPARATOR, true, true)));
    lss_test!(lss_long_l_before(["--long", "mydir"] => ("mydir", LIST_ENTRY_SEPARATOR, true, true)));
    lss_test!(lss_list_l_after(["mydir", "--list"] => ("mydir", LIST_ENTRY_SEPARATOR, true, true)));
    lss_test!(lss_list_l_before(["--list", "mydir"] => ("mydir", LIST_ENTRY_SEPARATOR, true, true)));
    lss_test!(lss_a(["-a"] => (DEFAULT_PATH, ENTRY_SEPARATOR, false, false)));
    lss_test!(lss_aa(["-A"] => (DEFAULT_PATH, ENTRY_SEPARATOR, false, true)));
    lss_test!(lss_implied_overwrite(["-aA"] => (DEFAULT_PATH, ENTRY_SEPARATOR, false, true)));
    lss_test!(lss_hidden_overwrite(["-A", "mydir", "-a"] => ("mydir", ENTRY_SEPARATOR, false, false)));
    lss_test!(lss_la(["mydir", "-la"] => ("mydir", LIST_ENTRY_SEPARATOR, false, false)));
    lss_test!(lss_aal(["-A", "mydir", "-l"] => ("mydir", LIST_ENTRY_SEPARATOR, false, true)));

    fn compare_dent_result(mut dents: Vec<String>, expected: &[&'static str]) {
        let mut expected = expected
            .iter()
            .map(|&s| String::from(s))
            .collect::<Vec<_>>();
        expected.sort_unstable();
        dents.sort_unstable();
        assert_eq!(expected, dents);
    }

    #[test_case]
    fn dent_names_empty_dir() {
        const PATH: &str = "/tmp/tlenix_ls_dent_names_empty_dir";
        fs::mkdir(PATH, fs::FilePermissions::from(0o755)).unwrap();
        let dn_result = dent_names(PATH);
        fs::rmdir(PATH).unwrap();
        compare_dent_result(dn_result.unwrap(), &[".", ".."][..]);
    }

    #[test_case]
    fn dent_names_full_dir() {
        const PATH: &str = "/tmp/tlenix_ls_dent_names_full_dir";
        const SUBDIR: &str = "subdir";
        const FILE_1: &str = "f1";
        const FILE_2: &str = "f2";
        let mut subdir_path = String::from(PATH);
        subdir_path.push('/');
        subdir_path.push_str(SUBDIR);
        let mut file1_path = String::from(PATH);
        file1_path.push('/');
        file1_path.push_str(FILE_1);
        let mut file2_path = String::from(PATH);
        file2_path.push('/');
        file2_path.push_str(FILE_2);

        fs::mkdir(PATH, fs::FilePermissions::from(0o755)).unwrap();
        fs::mkdir(subdir_path.as_str(), fs::FilePermissions::from(0o755)).unwrap();
        fs::OpenOptions::new()
            .create(true)
            .open(file1_path.as_str())
            .unwrap();
        fs::OpenOptions::new()
            .create(true)
            .open(file2_path.as_str())
            .unwrap();

        let dn_result = dent_names(PATH);

        fs::rm(file2_path).unwrap();
        fs::rm(file1_path).unwrap();
        fs::rmdir(subdir_path).unwrap();
        fs::rmdir(PATH).unwrap();

        compare_dent_result(dn_result.unwrap(), &[".", "..", "subdir", "f1", "f2"][..]);
    }
}
