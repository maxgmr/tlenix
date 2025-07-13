//! Simple game. Guess the code.

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

use alloc::{fmt::Display, string::String, vec::Vec};
use core::{error::Error, panic::PanicInfo, write};

use tlenix_core::{
    Console, EnvVar, Errno, eprintln, parse_argv_envp, print, println,
    process::{self, ExitStatus},
    try_exit,
};

const PANIC_TITLE: &str = "codebreaker";

const CODE_LEN: usize = 5;
const NUM_GUESSES: usize = 8;

/// Defines a const str with the given content and foreground colour.
macro_rules! define_coloured_str {
    ($name:ident($num:literal) = $string:literal) => {
        const $name: &str = concat!("\u{001b}[", $num, "m", $string, "\u{001b}[0m");
    };
}

define_coloured_str!(RED_STR(91) = "o");
define_coloured_str!(GREEN_STR(92) = "o");
define_coloured_str!(YELLOW_STR(93) = "o");
define_coloured_str!(BLUE_STR(94) = "o");
define_coloured_str!(PURPLE_STR(95) = "o");
define_coloured_str!(AQUA_STR(96) = "o");
define_coloured_str!(WHITE_STR(97) = "o");

define_coloured_str!(RIGHT_PLACE_STR(97) = "!");
define_coloured_str!(RIGHT_COLOUR_STR(37) = "?");
define_coloured_str!(NEITHER_STR(90) = ".");

core::arch::global_asm! {
    ".global _start",
    "_start:",
    "mov rdi, rsp",
    "call start"
}

/// Implementors of this trait can be formatted as either colour or monochrome static strings.
///
/// By default, the `as_colour_str` function uses the `as_mono_str` implementation.
trait AsColourOrMonoStr {
    /// Formats as a static monochrome str.
    fn as_mono_str(&self) -> &'static str;

    /// Formats as a static colour str.
    fn as_colour_str(&self) -> &'static str {
        self.as_mono_str()
    }

    /// Formats a slice of this type as a space-separated monochrome String.
    fn mono_list_string(elems: &[Self]) -> String
    where
        Self: Sized,
    {
        slice_fmt_helper(elems, Self::as_mono_str)
    }

    /// Formats a slice of this type as a space-separated coloured String.
    fn colour_list_string(elems: &[Self]) -> String
    where
        Self: Sized,
    {
        slice_fmt_helper(elems, Self::as_colour_str)
    }

    /// Formats a slice of this type as a space-separated String, colouring dependent on the
    /// provided boolean.
    fn list_string(elems: &[Self], in_colour: bool) -> String
    where
        Self: Sized,
    {
        if in_colour {
            Self::colour_list_string(elems)
        } else {
            Self::mono_list_string(elems)
        }
    }
}

// Helper for `AsColourOrMonoStr` slice formatting.
fn slice_fmt_helper<S, F>(elems: &[S], f: F) -> String
where
    S: AsColourOrMonoStr,
    F: FnMut(&S) -> &'static str,
{
    elems.iter().map(f).collect::<Vec<&'static str>>().join(" ")
}

/// A peg of a given colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Peg {
    Red,
    Green,
    Yellow,
    Blue,
    Purple,
    Aqua,
    White,
}
impl Peg {
    fn try_from_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "red" => Some(Self::Red),
            "green" => Some(Self::Green),
            "yellow" => Some(Self::Yellow),
            "blue" => Some(Self::Blue),
            "purple" => Some(Self::Purple),
            "aqua" => Some(Self::Aqua),
            "white" => Some(Self::White),
            s => Self::try_from_char(s.chars().next().unwrap_or('\0')),
        }
    }

    fn try_from_char(c: char) -> Option<Self> {
        match c.to_ascii_lowercase() {
            'r' => Some(Self::Red),
            'g' => Some(Self::Green),
            'y' => Some(Self::Yellow),
            'b' => Some(Self::Blue),
            'p' => Some(Self::Purple),
            'a' => Some(Self::Aqua),
            'w' => Some(Self::White),
            _ => None,
        }
    }

    fn as_word(self) -> &'static str {
        match self {
            Self::Red => "red",
            Self::Green => "green",
            Self::Yellow => "yellow",
            Self::Blue => "blue",
            Self::Purple => "purple",
            Self::Aqua => "aqua",
            Self::White => "white",
        }
    }

    fn iterator() -> impl Iterator<Item = Self> {
        [
            Self::Red,
            Self::Green,
            Self::Yellow,
            Self::Blue,
            Self::Purple,
            Self::Aqua,
            Self::White,
        ]
        .iter()
        .copied()
    }
}
impl AsColourOrMonoStr for Peg {
    fn as_mono_str(&self) -> &'static str {
        match self {
            Self::Red => "r",
            Self::Green => "g",
            Self::Yellow => "y",
            Self::Blue => "b",
            Self::Purple => "p",
            Self::Aqua => "a",
            Self::White => "w",
        }
    }

    fn as_colour_str(&self) -> &'static str {
        match self {
            Self::Red => RED_STR,
            Self::Green => GREEN_STR,
            Self::Yellow => YELLOW_STR,
            Self::Blue => BLUE_STR,
            Self::Purple => PURPLE_STR,
            Self::Aqua => AQUA_STR,
            Self::White => WHITE_STR,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CodeFromStrError {
    TooFew,
    TooMany,
    UnknownColour,
}
impl Display for CodeFromStrError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::TooFew => write!(f, "not enough colours given, expected {CODE_LEN}"),
            Self::TooMany => write!(f, "too many colours given, expected {CODE_LEN}"),
            Self::UnknownColour => write!(
                f,
                "unrecognized colour given, valid colours are {}",
                Peg::iterator()
                    .map(Peg::as_word)
                    .collect::<Vec<&'static str>>()
                    .join(", ")
            ),
        }
    }
}
impl Error for CodeFromStrError {}

/// A given code. A sequence of [`Peg`]s.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Code([Peg; CODE_LEN]);
impl Code {
    fn try_from_str(s: &str) -> Result<Self, CodeFromStrError> {
        let mut contents = [Peg::Red; CODE_LEN];

        if s.len() == CODE_LEN {
            // Interpret as a string of single-letter colour codes
            for (i, c) in s.chars().enumerate() {
                if let Some(pin) = Peg::try_from_char(c) {
                    // OK to index- `s` and `contents` are both pinned to `CODE_LEN`.
                    contents[i] = pin;
                } else {
                    return Err(CodeFromStrError::UnknownColour);
                }
            }
            // `CODE_LEN` valid colours have been parsed from the string. Return.
            return Ok(Self(contents));
        }

        // Interpret as whitespace-separated colours
        let str_list = s.split_whitespace().collect::<Vec<_>>();

        if str_list.len() < CODE_LEN {
            return Err(CodeFromStrError::TooFew);
        }

        if str_list.len() > CODE_LEN {
            return Err(CodeFromStrError::TooMany);
        }

        // List is valid length.
        for (i, elem) in str_list.iter().enumerate() {
            if let Some(pin) = Peg::try_from_str(elem) {
                // OK to index- we've already confirmed `str_list` is length `CODE_LEN`.
                contents[i] = pin;
            } else {
                return Err(CodeFromStrError::UnknownColour);
            }
        }

        Ok(Self(contents))
    }
}

/// A given piece of info.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Hint {
    RightPlace,
    RightColour,
    Neither,
}
impl AsColourOrMonoStr for Hint {
    fn as_mono_str(&self) -> &'static str {
        match self {
            Self::RightPlace => "!",
            Self::RightColour => "?",
            Self::Neither => ".",
        }
    }

    fn as_colour_str(&self) -> &'static str {
        match self {
            Self::RightPlace => RIGHT_PLACE_STR,
            Self::RightColour => RIGHT_COLOUR_STR,
            Self::Neither => NEITHER_STR,
        }
    }
}

/// A given sequence of hints. Gives info on how much of the player's guess is correct.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Feedback([Hint; CODE_LEN]);

/// The state of the game.
#[derive(Clone, Debug)]
struct GameState {
    actual_code: Code,
    guesses: [Option<Code>; NUM_GUESSES],
    in_colour: bool,
}
impl GameState {
    fn new(in_colour: bool) -> Self {
        // TODO generate random code
        Self {
            actual_code: Code([Peg::Red, Peg::Green, Peg::Yellow, Peg::Blue, Peg::Purple]),
            guesses: [None; NUM_GUESSES],
            in_colour,
        }
    }

    fn current_guess_number(&self) -> usize {
        let mut guess_num = 0;
        let mut guesses_iter = self.guesses.iter();
        while let Some(Some(_)) = guesses_iter.next() {
            guess_num += 1;
        }
        guess_num
    }

    fn remaining_guesses(&self) -> usize {
        NUM_GUESSES - self.current_guess_number()
    }

    fn get_feedback(&self, guess: Code) -> Feedback {
        let mut num_right_place = 0;
        let mut num_right_colour = 0;

        // Since this loop runs `CODE_LEN` times and can't increment `num_right_place` AND
        // `num_right_colour` at the same time, `num_right_place` + `num_right_colour` <=
        // `CODE_LEN` = `feedback.len()`
        for (i, peg) in self.actual_code.0.iter().enumerate() {
            if &guess.0[i] == peg {
                num_right_place += 1;
            } else if guess.0.contains(peg) {
                num_right_colour += 1;
            }
        }

        let mut feedback = Feedback([Hint::Neither; CODE_LEN]);
        // We already know that num_right_place + num_right_colour won't go out of bounds (see
        // comment above).
        for i in 0..num_right_place {
            feedback.0[i] = Hint::RightPlace;
        }
        for i in num_right_place..(num_right_place + num_right_colour) {
            feedback.0[i] = Hint::RightColour;
        }

        feedback
    }
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

fn welcome() {
    println!(
        "Welcome to Codebreaker! You must guess the secret pattern of {CODE_LEN} unique colours in {NUM_GUESSES} guesses or fewer."
    );
    println!(
        "Possible colours: {}",
        Peg::iterator()
            .map(Peg::as_word)
            .collect::<Vec<&'static str>>()
            .join(", ")
    );
    println!("------------------------");
}

fn read_guess() -> Result<Code, Errno> {
    // Keep trying to read a line until the user inputs something correct
    loop {
        print!("Please input your guess: ");
        let input = read_line()?;
        match Code::try_from_str(&input) {
            Ok(guess) => {
                return Ok(guess);
            }
            Err(e) => {
                eprintln!("{e}");
                eprintln!(
                    "Expected input: Either a list of colours (e.g. \"red green yellow blue purple\") or a length-{CODE_LEN} string of single-letter colours (e.g. \"rgybp\")."
                );
            }
        }
    }
}

fn response(game_state: &GameState, guess: Code) {
    print!(
        "Guess {} ({} remaining): ",
        game_state.current_guess_number() + 1,
        game_state.remaining_guesses() - 1,
    );
    println!(
        "{} | {}",
        Peg::list_string(&guess.0, game_state.in_colour),
        Hint::list_string(&game_state.get_feedback(guess).0, game_state.in_colour)
    );
}

// TODO: instead, turn off echo and only accept valid individual characters, printing to screen if
// character is valid
fn read_line() -> Result<String, Errno> {
    let console = Console::open()?;
    Ok(String::from_utf8_lossy(&console.read_line(64)?).into_owned())
}

fn main(_args: &[String], _env_vars: &[EnvVar]) -> ExitStatus {
    welcome();

    let mut game_state = GameState::new(true);

    while game_state.remaining_guesses() > 0 {
        let guess = try_exit!(read_guess());
        response(&game_state, guess);

        // Check if win
        if guess == game_state.actual_code {
            println!(
                "You cracked the code in {} guess(es)!",
                game_state.current_guess_number()
            );
            return ExitStatus::ExitSuccess;
        }

        game_state.guesses[game_state.current_guess_number()] = Some(guess);
    }

    // Out of guesses. Lost.
    println!("You ran out of guesses :(");
    println!(
        "Actual code: {}",
        Peg::list_string(&game_state.actual_code.0, game_state.in_colour)
    );

    ExitStatus::ExitSuccess
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
    fn gs_guess_num_remaining() {
        let mut gs = GameState::new(true);
        for i in 0..NUM_GUESSES {
            assert_eq!(gs.current_guess_number(), i);
            assert_eq!(gs.remaining_guesses(), NUM_GUESSES - i);
            gs.guesses[i] = Some(Code([Peg::Red; CODE_LEN]));
            assert_eq!(gs.current_guess_number(), i + 1);
        }
        assert_eq!(gs.remaining_guesses(), 0);
        assert_eq!(gs.current_guess_number(), NUM_GUESSES);
    }

    #[test_case]
    fn code_from_valid_list() {
        assert_eq!(
            Code::try_from_str("RED bluE a green W"),
            Ok(Code([
                Peg::Red,
                Peg::Blue,
                Peg::Aqua,
                Peg::Green,
                Peg::White
            ])),
        );
    }

    #[test_case]
    fn code_from_valid_single_chars() {
        assert_eq!(
            Code::try_from_str("wpyyg"),
            Ok(Code([
                Peg::White,
                Peg::Purple,
                Peg::Yellow,
                Peg::Yellow,
                Peg::Green,
            ]))
        );
    }

    #[test_case]
    fn code_from_other_whitespace_list() {
        assert_eq!(
            Code::try_from_str("r\tblue\t\tgreen YELLOW     aqua"),
            Ok(Code([
                Peg::Red,
                Peg::Blue,
                Peg::Green,
                Peg::Yellow,
                Peg::Aqua,
            ]))
        );
    }

    #[test_case]
    fn code_from_too_short_list() {
        assert_eq!(
            Code::try_from_str("red blue green yellow"),
            Err(CodeFromStrError::TooFew),
        );
    }

    #[test_case]
    fn code_from_too_long_list() {
        assert_eq!(
            Code::try_from_str("aqua white purple blue yellow green"),
            Err(CodeFromStrError::TooMany),
        );
    }

    #[test_case]
    fn code_from_incorrect_str_len() {
        assert_eq!(Code::try_from_str("rgybpa"), Err(CodeFromStrError::TooFew),);
    }

    #[test_case]
    fn code_from_list_with_commas() {
        assert_eq!(
            Code::try_from_str("aqua, GREEN, w, b, p"),
            Ok(Code([
                Peg::Aqua,
                Peg::Green,
                Peg::White,
                Peg::Blue,
                Peg::Purple
            ]))
        );
    }
}
