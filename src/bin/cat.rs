//! Concatenate. Copies each given file to standard output.

#![warn(
    missing_docs,
    missing_debug_implementations,
    rust_2018_idioms,
    clippy::all,
    clippy::pedantic,
    clippy::todo
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
    EnvVar, Errno, eprintln, format, fs, parse_argv_envp,
    process::{self, ExitStatus},
    streams, try_exit,
};

const PANIC_TITLE: &str = "cat";

/// If this symbol is an argument, it means "read from stdin".
const STDIN_SYMBOL: &str = "-";

const LINE_END_BYTE: u8 = b'$';
const NONPRINTING_BYTE_1: u8 = b'M';
const NONPRINTING_BYTE_2: u8 = b'-';

const HIGH_BIT: u8 = 0x80;

const CARET_NOTATION_FLIP_BIT: u8 = 0x40;

core::arch::global_asm! {
    ".global _start",
    "_start:",
    "mov rdi, rsp",
    "call start"
}

/// The arguments and options given to `cat`.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[allow(clippy::struct_excessive_bools)]
struct CatInputs {
    files: Vec<String>,
    /// Number all nonempty output lines, starting with 1.
    number_nonblank: bool,
    /// Display a '$' after the end of each line. The `\r\n` combination is shown as '^M$'.
    show_ends: bool,
    /// Number all output lines, starting with 1.
    number: bool,
    /// Suppress repeated adjacent blank lines; output just one line instead of several.
    squeeze_blank: bool,
    /// Display TAB characters as '^I'.
    show_tabs: bool,
    /// Display control characters (except for line feed and tab) using caret notation. Precede
    /// characters that have the high bit set with 'M-'.
    show_nonprinting: bool,
}
impl CatInputs {
    /// Applies the options to the given byte vector.
    fn apply(&self, bytes: &mut Vec<u8>) {
        if self.is_no_options() {
            return;
        }

        // Create a secondary buffer which replaces the original
        let mut result = Vec::with_capacity(bytes.len());

        let mut is_line_start = true;
        let mut last_line_blank = false;
        let mut line_num = 1;

        for &b in bytes.iter() {
            // It's the end of the line if the current character is the line feed.
            let is_line_end = b == b'\n';
            let is_line_blank = is_line_start && is_line_end;

            if self.squeeze_blank && is_line_blank && last_line_blank {
                continue;
            }

            if (self.number && is_line_start)
                || (self.number_nonblank && is_line_start && !is_line_blank)
            {
                Self::push_line_num(&mut result, line_num);
            }

            if self.show_ends && is_line_end {
                result.push(LINE_END_BYTE);
            }

            // Time to push the byte!
            if self.show_nonprinting && Self::is_high_bit_set(b) {
                result.push(NONPRINTING_BYTE_1);
                result.push(NONPRINTING_BYTE_2);
                // Reset high bit of b
                result.push(b & !HIGH_BIT);
            } else if self.should_show_nonprinting(b) {
                // `get_caret_notation_char` is safe to call because the conditional requires the
                // character to be an ASCII control character.
                Self::push_caret_notation_byte(&mut result, Self::get_caret_notation_byte(b));
            } else {
                result.push(b);
            }

            // Set values for the next byte.
            if is_line_end && (!self.number_nonblank || !is_line_blank) {
                line_num += 1;
            }
            last_line_blank = is_line_blank;
            is_line_start = is_line_end;
        }

        // Replace the original vector.
        *bytes = result;
    }

    /// Return `true` iff:
    /// - The show nonprinting option is enabled and `b` is an ASCII control character that is not
    ///   the tab or line feed codes
    /// - OR, the show ends option is enabled and `c` is the carriage return code
    /// - OR, [`Self::show_tabs`] is enabled and `c` is the tab code
    fn should_show_nonprinting(&self, b: u8) -> bool {
        (self.show_nonprinting && b.is_ascii_control() && (b != b'\t') && (b != b'\n'))
            || (self.show_ends && (b == b'\r'))
            || (self.show_tabs && (b == b'\t'))
    }

    fn push_line_num(bytes: &mut Vec<u8>, line_num: i32) {
        // Pad to 6 characters to match the GNU coreutils version of `cat`
        bytes.extend(format!("{:>6}\t", line_num).into_bytes());
    }

    fn get_caret_notation_byte(b: u8) -> u8 {
        b ^ CARET_NOTATION_FLIP_BIT
    }

    fn push_caret_notation_byte(bytes: &mut Vec<u8>, caret_notation_byte: u8) {
        bytes.push(b'^');
        bytes.push(caret_notation_byte);
    }

    fn is_high_bit_set(byte: u8) -> bool {
        (byte & HIGH_BIT) != 0
    }

    /// Returns `true` if no options are set.
    fn is_no_options(&self) -> bool {
        !self.number_nonblank
            && !self.show_ends
            && !self.number
            && !self.squeeze_blank
            && !self.show_tabs
            && !self.show_nonprinting
    }
}
impl TryFrom<&[String]> for CatInputs {
    type Error = Errno;
    fn try_from(value: &[String]) -> Result<Self, Self::Error> {
        let mut cat_inputs = Self::default();

        let mut opts = Options::new(value.iter().map(String::as_str).skip(1));
        while let Some(arg) = opts.next_arg().map_err(|_| Errno::Einval)? {
            match arg {
                Arg::Short('A') | Arg::Long("show-all") => {
                    cat_inputs.show_ends = true;
                    cat_inputs.show_tabs = true;
                    cat_inputs.show_nonprinting = true;
                }
                Arg::Short('b') | Arg::Long("number-nonblank") => {
                    cat_inputs.number_nonblank = true;
                    cat_inputs.number = false;
                }
                Arg::Short('e') => {
                    cat_inputs.show_ends = true;
                    cat_inputs.show_nonprinting = true;
                }
                Arg::Short('E') | Arg::Long("show-ends") => {
                    cat_inputs.show_ends = true;
                }
                Arg::Short('n') | Arg::Long("number") => {
                    if !cat_inputs.number_nonblank {
                        cat_inputs.number = true;
                    }
                }
                Arg::Short('s') | Arg::Long("squeeze-blank") => {
                    cat_inputs.squeeze_blank = true;
                }
                Arg::Short('t') => {
                    cat_inputs.show_tabs = true;
                    cat_inputs.show_nonprinting = true;
                }
                Arg::Short('T') | Arg::Long("show-tabs") => {
                    cat_inputs.show_tabs = true;
                }
                Arg::Short('v') | Arg::Long("show-nonprinting") => {
                    cat_inputs.show_nonprinting = true;
                }
                Arg::Positional(file) => cat_inputs.files.push(file.to_string()),
                _ => {}
            }
        }
        Ok(cat_inputs)
    }
}

/// Concatenate. Copies each file to standard output.
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
    let cat_inputs = try_exit!(CatInputs::try_from(args));

    let mut output = try_exit!(concatenate(&cat_inputs.files));

    // Apply options to output
    cat_inputs.apply(&mut output);

    // Output to stdout
    try_exit!(streams::STDOUT.lock().write(&output));

    ExitStatus::ExitSuccess
}

fn concatenate(files: &[String]) -> Result<Vec<u8>, Errno> {
    let mut output = Vec::new();

    // If empty, get stdin
    if files.is_empty() {
        append_stdin_bytes(&mut output)?;
    } else
    // Read input from files
    {
        for file in files {
            if file == STDIN_SYMBOL {
                append_stdin_bytes(&mut output)?;
            } else {
                append_file_bytes(&mut output, file)?;
            }
        }
    }

    Ok(output)
}

/// Appends standard input to a vector of bytes.
fn append_stdin_bytes(buf: &mut Vec<u8>) -> Result<(), Errno> {
    buf.append(&mut streams::STDIN.lock().read_to_bytes()?);
    Ok(())
}

/// Appends the file bytes to a vector of bytes.
fn append_file_bytes(buf: &mut Vec<u8>, path: &str) -> Result<(), Errno> {
    buf.append(&mut fs::OpenOptions::new().open(path)?.read_to_bytes()?);
    Ok(())
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    eprintln!("{PANIC_TITLE} {info}");
    process::exit(ExitStatus::ExitFailure(1))
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;

    const CAT_TEST_DIR: &str = "/tmp/tlenix_cat_tests";

    macro_rules! cat_inputs_test {
        ($fn_name:ident[$($arg:expr),*] => CatInputs {
            $(files: [$($ex_f:expr),*],)?
            $(number_nonblank: $ex_nnb:expr,)?
            $(show_ends: $ex_se:expr,)?
            $(number: $ex_n:expr,)?
            $(squeeze_blank: $ex_sb:expr,)?
            $(show_tabs: $ex_st:expr,)?
            $(show_nonprinting: $ex_snp:expr,)?
        }) => {
           #[test_case]
           fn $fn_name() {
               let input: &[String] = &["cat".to_string(), $($arg.to_string()),*];
               let ex = CatInputs::try_from(input).unwrap();
               $(
                   let files: &[String] = &[$($ex_f.to_string()),*];
                   assert_eq!(ex.files, files);
                )?
                $(assert_eq!(ex.number_nonblank, $ex_nnb);)?
                $(assert_eq!(ex.show_ends, $ex_se);)?
                $(assert_eq!(ex.number, $ex_n);)?
                $(assert_eq!(ex.squeeze_blank, $ex_sb);)?
                $(assert_eq!(ex.show_tabs, $ex_st);)?
                $(assert_eq!(ex.show_nonprinting, $ex_snp);)?
           }
        };
    }
    cat_inputs_test!(empty[] => CatInputs {
        files: [],
    });
    cat_inputs_test!(stdin_only[STDIN_SYMBOL] => CatInputs {
       files: [STDIN_SYMBOL],
    });
    cat_inputs_test!(multiple_files["f1", "./test/f2.txt", "/root/my_file"] => CatInputs {
        files: ["f1", "./test/f2.txt", "/root/my_file"],
    });
    cat_inputs_test!(files_and_stdins["f1", STDIN_SYMBOL, "f2", STDIN_SYMBOL] => CatInputs {
        files: ["f1", STDIN_SYMBOL, "f2", STDIN_SYMBOL],
    });
    cat_inputs_test!(interspersed_options["-A", "-", "--squeeze-blank", "f1", "-Z"] => CatInputs {
        files: ["-", "f1"],
        number_nonblank: false,
        show_ends: true,
        number: false,
        squeeze_blank: true,
        show_tabs: true,
        show_nonprinting: true,
    });
    cat_inputs_test!(number_nonblank_override["--number", "-b"] => CatInputs {
        number_nonblank: true,
        number: false,
    });
    cat_inputs_test!(number_overridden["--number-nonblank", "-n"] => CatInputs {
        number_nonblank: true,
        number: false,
    });
    cat_inputs_test!(show_all_short["-A"] => CatInputs {
        number_nonblank: false,
        show_ends: true,
        number: false,
        squeeze_blank: false,
        show_tabs: true,
        show_nonprinting: true,
    });
    cat_inputs_test!(show_all_long["--show-all"] => CatInputs {
        number_nonblank: false,
        show_ends: true,
        number: false,
        squeeze_blank: false,
        show_tabs: true,
        show_nonprinting: true,
    });
    cat_inputs_test!(number_nonblank_short["-b"] => CatInputs {
        number_nonblank: true,
        show_ends: false,
        number: false,
        squeeze_blank: false,
        show_tabs: false,
        show_nonprinting: false,
    });
    cat_inputs_test!(number_nonblank_long["--number-nonblank"] => CatInputs {
        number_nonblank: true,
        show_ends: false,
        number: false,
        squeeze_blank: false,
        show_tabs: false,
        show_nonprinting: false,
    });
    cat_inputs_test!(ve["-e"] => CatInputs {
        number_nonblank: false,
        show_ends: true,
        number: false,
        squeeze_blank: false,
        show_tabs: false,
        show_nonprinting: true,
    });
    cat_inputs_test!(show_ends_short["-E"] => CatInputs {
        number_nonblank: false,
        show_ends: true,
        number: false,
        squeeze_blank: false,
        show_tabs: false,
        show_nonprinting: false,
    });
    cat_inputs_test!(show_ends_long["--show-ends"] => CatInputs {
        number_nonblank: false,
        show_ends: true,
        number: false,
        squeeze_blank: false,
        show_tabs: false,
        show_nonprinting: false,
    });
    cat_inputs_test!(number_short["-n"] => CatInputs {
        number_nonblank: false,
        show_ends: false,
        number: true,
        squeeze_blank: false,
        show_tabs: false,
        show_nonprinting: false,
    });
    cat_inputs_test!(number_long["--number"] => CatInputs {
        number_nonblank: false,
        show_ends: false,
        number: true,
        squeeze_blank: false,
        show_tabs: false,
        show_nonprinting: false,
    });
    cat_inputs_test!(squeeze_blank_short["-s"] => CatInputs {
        number_nonblank: false,
        show_ends: false,
        number: false,
        squeeze_blank: true,
        show_tabs: false,
        show_nonprinting: false,
    });
    cat_inputs_test!(squeeze_blank_long["--squeeze-blank"] => CatInputs {
        number_nonblank: false,
        show_ends: false,
        number: false,
        squeeze_blank: true,
        show_tabs: false,
        show_nonprinting: false,
    });
    cat_inputs_test!(vt["-t"] => CatInputs {
        number_nonblank: false,
        show_ends: false,
        number: false,
        squeeze_blank: false,
        show_tabs: true,
        show_nonprinting: true,
    });
    cat_inputs_test!(show_tabs_short["-T"] => CatInputs {
        number_nonblank: false,
        show_ends: false,
        number: false,
        squeeze_blank: false,
        show_tabs: true,
        show_nonprinting: false,
    });
    cat_inputs_test!(show_tabs_long["--show-tabs"] => CatInputs {
        number_nonblank: false,
        show_ends: false,
        number: false,
        squeeze_blank: false,
        show_tabs: true,
        show_nonprinting: false,
    });
    cat_inputs_test!(show_nonprinting_short["-v"] => CatInputs {
        number_nonblank: false,
        show_ends: false,
        number: false,
        squeeze_blank: false,
        show_tabs: false,
        show_nonprinting: true,
    });
    cat_inputs_test!(show_nonprinting_long["--show-nonprinting"] => CatInputs {
        number_nonblank: false,
        show_ends: false,
        number: false,
        squeeze_blank: false,
        show_tabs: false,
        show_nonprinting: true,
    });

    #[test_case]
    fn check_concatenate() {
        const FILES: [&str; 3] = [
            "test_concatenate1",
            "test_concatenate2",
            "test_concatenate3",
        ];

        const CONTENTS: [&str; 3] = ["abc\n马克斯\n", "123\n", "def\n456\n"];
        const EXPECTED: &str = "abc\n马克斯\n123\ndef\n456\n";

        let paths: Vec<String> = FILES
            .iter()
            .map(|f| format!("{CAT_TEST_DIR}/{f}"))
            .collect();

        let _ = fs::mkdir(CAT_TEST_DIR, fs::FilePermissions::from(0o777));
        for i in 0..FILES.len() {
            fs::OpenOptions::new()
                .read_write()
                .create(true)
                .open(paths[i].as_str())
                .unwrap()
                .write(CONTENTS[i].as_bytes())
                .unwrap();
        }

        let concat_result = concatenate(&paths);

        // Clean up after yourself
        for path in paths {
            fs::rm(path).unwrap();
        }
        fs::rmdir(CAT_TEST_DIR).unwrap();

        assert_eq!(concat_result.unwrap(), EXPECTED.as_bytes());
    }

    fn opts_test(mut input: Vec<u8>, cat_inputs: &CatInputs, expected: &[u8]) {
        cat_inputs.apply(&mut input);
        // tlenix_core::print!("\nRESULT\n{}", core::str::from_utf8(&input).unwrap());
        // tlenix_core::print!("\nEXPECTED\n{}", core::str::from_utf8(expected).unwrap());
        assert_eq!(&input, expected);
    }

    #[test_case]
    fn number_nonblank() {
        let mut cat_inputs = CatInputs::default();
        cat_inputs.number_nonblank = true;
        let orig = r"a
b

c


d
e
f
g
h
i
j
k
";
        let expected = r"     1	a
     2	b

     3	c


     4	d
     5	e
     6	f
     7	g
     8	h
     9	i
    10	j
    11	k
";
        opts_test(orig.as_bytes().to_vec(), &cat_inputs, expected.as_bytes());
    }

    #[test_case]
    fn show_ends() {
        let mut cat_inputs = CatInputs::default();
        cat_inputs.show_ends = true;
        let input = "a\nb\r\nc\n";
        let expected = "a$\nb^M$\nc$\n";
        opts_test(input.as_bytes().to_vec(), &cat_inputs, expected.as_bytes());
    }

    #[test_case]
    fn number() {
        let mut cat_inputs = CatInputs::default();
        cat_inputs.number = true;
        let orig = r"a
b

c


d
e
f
g
h
";
        let expected = r"     1	a
     2	b
     3	
     4	c
     5	
     6	
     7	d
     8	e
     9	f
    10	g
    11	h
";
        opts_test(orig.as_bytes().to_vec(), &cat_inputs, expected.as_bytes());
    }

    #[test_case]
    fn squeeze_blank() {
        let mut cat_inputs = CatInputs::default();
        cat_inputs.squeeze_blank = true;
        let orig = r"a
b

c


d



e
f
";
        let expected = r"a
b

c

d

e
f
";
        opts_test(orig.as_bytes().to_vec(), &cat_inputs, expected.as_bytes());
    }

    #[test_case]
    fn show_tabs() {
        let mut cat_inputs = CatInputs::default();
        cat_inputs.show_tabs = true;
        let orig = "a\tb\t\tc\t\t\td\r\ne\t\n";
        let expected = "a^Ib^I^Ic^I^I^Id\r\ne^I\n";
        opts_test(orig.as_bytes().to_vec(), &cat_inputs, expected.as_bytes());
    }

    #[test_case]
    fn show_nonprinting() {
        let mut cat_inputs = CatInputs::default();
        cat_inputs.show_nonprinting = true;
        opts_test(
            [
                (HIGH_BIT | b'x'),
                0x00,
                0x01,
                0x02,
                0x03,
                0x04,
                0x7F,
                b'\r',
                b'\n',
                b'\t',
            ]
            .to_vec(),
            &cat_inputs,
            "M-x^@^A^B^C^D^?^M\n\t".as_bytes(),
        );
    }
}
