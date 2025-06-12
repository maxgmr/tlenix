//! Moves a file from one place to another.

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
    Console, EnvVar, Errno, eprintln,
    fs::{self, FileStats, FileType},
    parse_argv_envp, print, println,
    process::{self, ExitStatus},
    try_exit,
};

const PANIC_TITLE: &str = "mv";

core::arch::global_asm! {
    ".global _start",
    "_start:",
    "mov rdi, rsp",
    "call start"
}

/// All the things that govern `mv`'s behaviour.
#[derive(Debug)]
struct MvSettings<'a> {
    paths: Vec<&'a str>,
    verbose: bool,
    rename_flags: fs::RenameFlags,
    prompt_overwrite: bool,
}
impl<'a> MvSettings<'a> {
    fn from_cli(args: &'a [String]) -> Result<Self, Errno> {
        let mut result = Self::default();

        let mut opts = Options::new(args.iter().map(String::as_str).skip(1));
        while let Some(arg) = opts.next_arg().map_err(|_| Errno::Einval)? {
            match arg {
                Arg::Short('v') | Arg::Long("debug") => {
                    tlenix_core::println!("v");
                    result.verbose = true;
                }
                Arg::Short('f') | Arg::Long("force") => {
                    tlenix_core::println!("f");
                    result.prompt_overwrite = false;
                    result.rename_flags.remove(fs::RenameFlags::NOREPLACE);
                }
                Arg::Short('n') | Arg::Long("no-clobber") => {
                    tlenix_core::println!("n");
                    result.prompt_overwrite = false;
                    result.rename_flags.insert(fs::RenameFlags::NOREPLACE);
                    result.rename_flags.remove(fs::RenameFlags::EXCHANGE);
                }
                Arg::Short('i') | Arg::Long("interactive") => {
                    tlenix_core::println!("i");
                    result.prompt_overwrite = true;
                    result.rename_flags.remove(fs::RenameFlags::NOREPLACE);
                }
                Arg::Long("exchange") => {
                    tlenix_core::println!("exchange");
                    result.rename_flags.insert(fs::RenameFlags::EXCHANGE);
                    result.rename_flags.remove(fs::RenameFlags::NOREPLACE);
                }
                Arg::Positional(value) => {
                    result.paths.push(value);
                }
                _ => {}
            }
        }

        Ok(result)
    }
}
impl Default for MvSettings<'_> {
    fn default() -> Self {
        Self {
            paths: Vec::new(),
            verbose: false,
            rename_flags: fs::RenameFlags::empty(),
            prompt_overwrite: false,
        }
    }
}

/// Move a file from one place to another.
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
    let settings = try_exit!(MvSettings::from_cli(args));
    let mut _stdin = String::new();
    if settings.paths.len() < 2 {
        eprintln!("Usage: 'mv <source> <destination>'");
        return ExitStatus::ExitFailure(255);
    }

    try_exit!(move_files(&settings));

    ExitStatus::ExitSuccess
}

fn move_files(settings: &MvSettings<'_>) -> Result<(), Errno> {
    if settings.paths.len() < 2 {
        return Err(Errno::Einval);
    }

    // SAFE: Accessing based on the length of the slice itself. We already ensured the length was 2
    // or greater.
    let dest_path = settings.paths[settings.paths.len() - 1];
    let dest_stats = FileStats::try_from_path(dest_path).ok();
    let dest_type = if let Some(stats) = dest_stats {
        Some(stats.file_type.ok_or(Errno::Ebadf)?)
    } else {
        None
    };

    if settings.paths.len() == 2 {
        // Moving a single thing.
        // SAFE: We just checked that the length was 2.
        let source_path = settings.paths[0];

        let source_file_stats = FileStats::try_from_path(source_path).inspect_err(|&e| {
            if e == Errno::Enoent {
                eprintln!("mv failed: Source '{source_path}' does not exist");
            }
        })?;

        match (source_file_stats.file_type.ok_or(Errno::Ebadf)?, dest_type) {
            (_, Some(FileType::Directory)) => {
                // Destination is a directory. Move the file inside the directory.
                return move_file_inside_directory(source_path, dest_path, settings);
            }
            (FileType::Directory, Some(_)) => {
                // Source is a directory. Destination isn't a directory. Fail.
                return Err(Errno::Enotdir);
            }
            _ => {
                // Rename the file, overwriting the destination if it exists.
                return rename_with_settings(source_path, dest_path, settings);
            }
        }
    }

    // More than two args. We're moving multiple files.
    // If the destination isn't a directory, fail.
    if dest_type != Some(FileType::Directory) {
        return Err(Errno::Enotdir);
    }

    // Move all the files inside the destination directory.
    for &arg in settings.paths.iter().take(settings.paths.len() - 1) {
        move_file_inside_directory(arg, dest_path, settings)?;
    }
    Ok(())
}

fn get_file_name(path: &str) -> Option<&str> {
    // Trim trailing slashes
    let trimmed_path = path.trim_end_matches('/');

    // Split on '/' and filter out empty parts
    let mut parts = trimmed_path.split('/').filter(|&s| !s.is_empty());

    // Get the last non-empty part (if any)
    let last_part = parts.next_back();

    // Only return if it's not "." or ".."
    match last_part {
        Some("." | "..") | None => None,
        Some(name) => Some(name),
    }
}

/// Returns [`Errno::Einval`] if `file_path` doesn't point to a file.
fn move_file_inside_directory(
    file_path: &str,
    dir_path: &str,
    settings: &MvSettings<'_>,
) -> Result<(), Errno> {
    let dest = dir_path.to_string() + "/" + get_file_name(file_path).ok_or(Errno::Einval)?;
    rename_with_settings(file_path, &dest, settings)
}

fn rename_with_settings(
    source: &str,
    destination: &str,
    settings: &MvSettings<'_>,
) -> Result<(), Errno> {
    // Check if prompt overwrite is enabled AND if a file exists at the destination.
    if settings.prompt_overwrite && FileStats::try_from_path(destination).is_ok() {
        let console = Console::open()?;
        print!("Overwrite '{destination}'? [y/N] ");
        match String::from_utf8(console.read_line(4096)?)
            .map_err(|_| Errno::Einval)?
            .to_lowercase()
            .as_str()
        {
            "yes" | "y" => {}
            _ => {
                return Ok(());
            }
        }
    }
    fs::rename(source, destination, settings.rename_flags)?;
    if settings.verbose {
        println!("Renamed '{source}' to '{destination}'.");
    }
    Ok(())
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    eprintln!("{PANIC_TITLE} {info}");
    process::exit(ExitStatus::ExitFailure(1))
}

#[cfg(test)]
mod tests {
    use tlenix_core::fs::OpenOptions;

    use super::*;

    const MV_TEST_DIR: &str = "/tmp/tlenix_mv_test_dir";

    // Helpers for different tests

    fn test_setup(test_name: &'static str) -> String {
        let main_dir = MV_TEST_DIR.to_string() + "/" + test_name;
        let _ = fs::mkdir(MV_TEST_DIR, fs::FilePermissions::from(0o777));
        let _ = fs::mkdir(&main_dir, fs::FilePermissions::from(0o777));
        main_dir
    }

    fn test_teardown(main_dir: &str) {
        let _ = fs::rmdir(main_dir);
        let _ = fs::rmdir(MV_TEST_DIR);
    }

    fn assert_exists(path: &str, expected_type: fs::FileType) {
        let stats = fs::FileStats::try_from_path(path).unwrap();
        assert_eq!(stats.file_type, Some(expected_type));
    }

    fn assert_dne(path: &str) {
        assert_eq!(fs::FileStats::try_from_path(path), Err(Errno::Enoent));
    }

    fn create_file_with_contents(path: &str, contents: &str) {
        let f = OpenOptions::new()
            .read_write()
            .create(true)
            .open(path)
            .unwrap();
        f.write(contents.as_bytes()).unwrap();
    }

    fn assert_contents(path: &str, expected: &str) {
        let f = OpenOptions::new().open(path).unwrap();
        assert_eq!(&f.read_to_string().unwrap(), expected);
    }

    fn dir_contains(dir_path: &str, file_path: &str) -> bool {
        let f_name = get_file_name(file_path).unwrap();
        fs::OpenOptions::new()
            .open(dir_path)
            .unwrap()
            .dir_ents()
            .unwrap()
            .iter()
            .any(|dent| dent.name.as_str() == f_name)
    }

    #[allow(clippy::field_reassign_with_default)]
    fn mk_mv_settings<'a>(paths: &'a [&str]) -> MvSettings<'a> {
        let mut result = MvSettings::default();
        result.paths = paths.to_vec();
        result
    }

    #[test_case]
    fn get_file_name_check() {
        let test_cases = [
            ("/some/dir/file.txt", Some("file.txt")),
            ("/path/to/dir/", Some("dir")),
            ("/multiple//slashes.txt", Some("slashes.txt")),
            ("./config.txt", Some("config.txt")),
            (".", None),
            ("..", None),
            ("", None),
            ("/", None),
        ];

        for (path, expected) in test_cases {
            assert_eq!(get_file_name(path), expected);
        }
    }

    #[test_case]
    fn file_to_new_name() {
        let dir_path = test_setup("file_to_new_name");

        let f_path = dir_path.clone() + "f1";
        let expected_path = dir_path.clone() + "f2";

        fs::OpenOptions::new().create(true).open(&f_path).unwrap();

        assert_exists(&f_path, FileType::RegularFile);
        assert_dne(&expected_path);

        // Move the file
        fs::rename(&f_path, &expected_path, fs::RenameFlags::empty()).unwrap();

        assert_exists(&expected_path, FileType::RegularFile);
        assert_dne(&f_path);

        fs::rm(&expected_path).unwrap();

        test_teardown(&dir_path);
    }

    #[test_case]
    fn overwrite_existing_file() {
        let dir_path = test_setup("overwrite_existing_file");

        let f1_path = dir_path.clone() + "/f1";
        let f2_path = dir_path.clone() + "/f2";

        let f1_contents = "123";
        let f2_contents = "abc";

        create_file_with_contents(&f1_path, f1_contents);
        create_file_with_contents(&f2_path, f2_contents);

        assert_contents(&f1_path, f1_contents);
        assert_contents(&f2_path, f2_contents);

        let args = [f1_path.as_str(), f2_path.as_str()];
        move_files(&mk_mv_settings(&args)).unwrap();

        assert_dne(&f1_path);
        assert_exists(&f2_path, FileType::RegularFile);
        assert_contents(&f2_path, f1_contents);

        fs::rm(&f2_path).unwrap();
        test_teardown(&dir_path);
    }

    #[test_case]
    fn move_file_into_dir() {
        let dir_path = test_setup("move_file_into_dir");

        let f_path = dir_path.clone() + "/f";
        let d_path = dir_path.clone() + "/d";
        let expected_path = dir_path.clone() + "/d/f";

        let f_contents = "123";

        create_file_with_contents(&f_path, f_contents);
        fs::mkdir(&d_path, fs::FilePermissions::from(0o777)).unwrap();

        assert_exists(&f_path, FileType::RegularFile);
        assert_exists(&d_path, FileType::Directory);
        assert_contents(&f_path, "123");
        assert!(!dir_contains(&d_path, &expected_path));

        let args = [f_path.as_str(), d_path.as_str()];
        move_files(&mk_mv_settings(&args)).unwrap();

        assert_dne(&f_path);

        assert_contents(&expected_path, f_contents);
        assert!(dir_contains(&d_path, &expected_path));

        fs::rm(&expected_path).unwrap();
        fs::rmdir(&d_path).unwrap();
        test_teardown(&dir_path);
    }

    #[test_case]
    fn dir_to_new_name() {
        let dir_path = test_setup("dir_to_new_name");

        let d_path = dir_path.clone() + "/d1";
        let f_path = d_path.clone() + "/f";
        let exp_d_path = dir_path.clone() + "/d2";
        let exp_f_path = exp_d_path.clone() + "/f";

        let f_contents = "123";

        fs::mkdir(&d_path, fs::FilePermissions::from(0o777)).unwrap();
        create_file_with_contents(&f_path, f_contents);

        assert_exists(&d_path, FileType::Directory);
        assert_exists(&f_path, FileType::RegularFile);
        assert_dne(&exp_d_path);
        assert_dne(&exp_f_path);
        assert_contents(&f_path, f_contents);

        let args = [d_path.as_str(), exp_d_path.as_str()];
        move_files(&mk_mv_settings(&args)).unwrap();

        assert_dne(&d_path);
        assert_dne(&f_path);
        assert_exists(&exp_d_path, FileType::Directory);
        assert_exists(&exp_f_path, FileType::RegularFile);
        assert_contents(&exp_f_path, f_contents);
        assert!(dir_contains(&exp_d_path, &exp_f_path));

        fs::rm(&exp_f_path).unwrap();
        fs::rmdir(&exp_d_path).unwrap();
        test_teardown(&dir_path);
    }

    #[test_case]
    fn dir_into_existing_dir() {
        let dir_path = test_setup("dir_into_existing_dir");

        let d1_path = dir_path.clone() + "/d1";
        let d2_path = dir_path.clone() + "/d2";
        let exp_d1_path = d2_path.clone() + "/d1";

        fs::mkdir(&d1_path, fs::FilePermissions::from(0o777)).unwrap();
        fs::mkdir(&d2_path, fs::FilePermissions::from(0o777)).unwrap();

        assert_exists(&d1_path, FileType::Directory);
        assert_exists(&d2_path, FileType::Directory);
        assert_dne(&exp_d1_path);

        let args = [d1_path.as_str(), d2_path.as_str()];
        move_files(&mk_mv_settings(&args)).unwrap();

        assert_dne(&d1_path);
        assert_exists(&d2_path, FileType::Directory);
        assert_exists(&exp_d1_path, FileType::Directory);
        assert!(dir_contains(&d2_path, &exp_d1_path));

        fs::rmdir(&exp_d1_path).unwrap();
        fs::rmdir(&d2_path).unwrap();
        test_teardown(&dir_path);
    }

    #[test_case]
    fn dir_into_file_fails() {
        let dir_path = test_setup("dir_into_file_fails");

        let d_path = dir_path.clone() + "/d";
        let f_path = dir_path.clone() + "/f";

        fs::mkdir(&d_path, fs::FilePermissions::from(0o777)).unwrap();
        create_file_with_contents(&f_path, "");

        let args = [d_path.as_str(), f_path.as_str()];
        assert_eq!(move_files(&mk_mv_settings(&args)), Err(Errno::Enotdir));

        fs::rm(&f_path).unwrap();
        fs::rmdir(&d_path).unwrap();
        test_teardown(&dir_path);
    }

    #[test_case]
    fn nonexistent_src_fails() {
        let args = ["fwliueghwgeuhjfhlfh3gg", "/tmp"];
        assert_eq!(move_files(&mk_mv_settings(&args)), Err(Errno::Enoent));
    }

    #[test_case]
    fn exchange_files() {
        let dir_path = test_setup("exchange_files");

        let f1 = dir_path.clone() + "/f1";
        let f1_contents = "111";
        let f2 = dir_path.clone() + "/f2";
        let f2_contents = "222";

        let f1_expected = f2.clone();
        let f2_expected = f1.clone();

        create_file_with_contents(&f1, f1_contents);
        create_file_with_contents(&f2, f2_contents);

        let args = [f1.as_str(), f2.as_str()];
        let mut mvs = mk_mv_settings(&args);
        mvs.rename_flags |= fs::RenameFlags::EXCHANGE;

        assert_exists(&f1, fs::FileType::RegularFile);
        assert_exists(&f2, fs::FileType::RegularFile);

        move_files(&mvs).unwrap();

        assert_contents(&f1_expected, f1_contents);
        assert_contents(&f2_expected, f2_contents);

        fs::rm(&f1_expected).unwrap();
        fs::rm(&f2_expected).unwrap();
        test_teardown(&dir_path);
    }

    #[test_case]
    fn no_clobber_stops_overwrite() {
        let dir_path = test_setup("no_clobber_stops_overwrite");

        let f1 = dir_path.clone() + "/f1";
        let f2 = dir_path.clone() + "/f2";
        let f1_contents = "123";
        let f2_contents = "abc";
        create_file_with_contents(&f1, f1_contents);
        create_file_with_contents(&f2, f2_contents);

        assert_exists(&f1, fs::FileType::RegularFile);
        assert_contents(&f1, f1_contents);
        assert_exists(&f2, fs::FileType::RegularFile);
        assert_contents(&f2, f2_contents);

        let args = [f1.as_str(), f2.as_str()];
        let mut mvs = mk_mv_settings(&args);
        mvs.rename_flags |= fs::RenameFlags::NOREPLACE;

        assert_eq!(move_files(&mvs), Err(Errno::Eexist));

        assert_exists(&f1, fs::FileType::RegularFile);
        assert_contents(&f1, f1_contents);
        assert_exists(&f2, fs::FileType::RegularFile);
        assert_contents(&f2, f2_contents);

        fs::rm(&f1).unwrap();
        fs::rm(&f2).unwrap();
        test_teardown(&dir_path);
    }

    #[test_case]
    fn settings_from_cli() {
        let args = [
            "mv".to_string(),
            "--debug".to_string(),
            "abc".to_string(),
            "-i".to_string(),
            "def".to_string(),
            "--exchange".to_string(),
            "--schmoop".to_string(),
        ];
        let expected = MvSettings {
            paths: [args[2].as_str(), args[4].as_str()].to_vec(),
            verbose: true,
            rename_flags: fs::RenameFlags::EXCHANGE,
            prompt_overwrite: true,
        };
        let result = MvSettings::from_cli(&args).unwrap();

        for (exp_path, res_path) in expected.paths.iter().zip(result.paths.iter()) {
            assert_eq!(exp_path, res_path);
        }
        assert_eq!(expected.verbose, result.verbose);
        assert_eq!(expected.rename_flags, result.rename_flags);
        assert_eq!(expected.prompt_overwrite, result.prompt_overwrite);
    }

    #[test_case]
    fn interactive_force_noclobber_overwrite() {
        let args = ["mv".to_string(), "-ifn".to_string()];
        let settings = MvSettings::from_cli(&args).unwrap();
        assert_eq!(settings.rename_flags, fs::RenameFlags::NOREPLACE);
        assert!(!settings.prompt_overwrite);

        let args = ["mv".to_string(), "-nif".to_string()];
        let settings = MvSettings::from_cli(&args).unwrap();
        assert_eq!(settings.rename_flags, fs::RenameFlags::empty());
        assert!(!settings.prompt_overwrite);

        let args = ["mv".to_string(), "-fni".to_string()];
        let settings = MvSettings::from_cli(&args).unwrap();
        assert_eq!(settings.rename_flags, fs::RenameFlags::empty());
        assert!(settings.prompt_overwrite);
    }
}
