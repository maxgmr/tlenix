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

use alloc::{
    fmt::Display,
    string::{String, ToString},
    vec::Vec,
};
use core::{error::Error, panic::PanicInfo, write};

use lazy_static::lazy_static;
use tlenix_core::{
    Console, EnvVar, Errno, eprintln, format, parse_argv_envp, print, println,
    process::{self, ExitStatus},
    streams::STDIN,
    term::{
        ControlCharIndex, ControlModeFlags, InputModeFlags, LocalModeFlags, OutputModeFlags,
        SetTermAttrsCmd, Termios,
    },
    try_exit,
};

const LOGO: &str = r"    __   ___   ___      ___  ____   ____     ___   ____  __  _    ___  ____  
   /  ] /   \ |   \    /  _]|    \ |    \   /  _] /    ||  |/ ]  /  _]|    \ 
  /  / |     ||    \  /  [_ |  o  )|  D  ) /  [_ |  o  ||  ' /  /  [_ |  D  )
 /  /  |  O  ||  D  ||    _]|     ||    / |    _]|     ||    \ |    _]|    / 
/   \_ |     ||     ||   [_ |  O  ||    \ |   [_ |  _  ||     \|   [_ |    \ 
\     ||     ||     ||     ||     ||  .  \|     ||  |  ||  .  ||     ||  .  \
 \____| \___/ |_____||_____||_____||__|\_||_____||__|__||__|\_||_____||__|\_|";

const PANIC_TITLE: &str = "codebreaker";

const CODE_LEN: usize = 5;
const NUM_GUESSES: usize = 8;

const PEG_SYMBOL: &str = "o";
const NO_PEG_SYMBOL: &str = ".";
const RIGHT_PLACE_SYMBOL: &str = "!";
const RIGHT_COLOUR_SYMBOL: &str = "?";
const NEITHER_SYMBOL: &str = ".";

/// ANSI escape code to clear the entire screen.
const CLEAR_SCREEN: &str = "\u{001b}[2J";
/// ANSI escape code to move the cursor to the top-left corner.
const CURSOR_TOP_LEFT: &str = "\u{001b}[H";

const READ_MIN_BYTES_READ: u8 = 0;
/// In deciseconds
const READ_MAX_TIME_PASSED: u8 = 1;

// CONTROLS
const EXIT_CODE: u8 = ctrl_key(b'q');
const ENTER_CODE: u8 = 0x0d;
const BACKSP_CODE: u8 = 0x7f;

const fn ctrl_key(byte: u8) -> u8 {
    byte & 0x1f
}

lazy_static! {
    static ref COLOURS_LIST_MONO: String = Peg::iterator()
        .map(|peg| format!("{} ({})", peg.as_word(), peg.as_letter()))
        .collect::<Vec<String>>()
        .join(", ");
    static ref COLOURS_LIST_COLOUR: String = Peg::iterator()
        .map(|peg| format!("{} ({})", peg.as_word_colour(), peg.as_letter_colour()))
        .collect::<Vec<String>>()
        .join(", ");
}

core::arch::global_asm! {
    ".global _start",
    "_start:",
    "mov rdi, rsp",
    "call start"
}

/// General helper function to format a string with a colour
fn ansi_colour_string_helper(s: &str, colour_code: usize) -> String {
    format!("\u{001b}[{}m{}\u{001b}[0m", colour_code, s)
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

    const fn as_word(self) -> &'static str {
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

    fn as_word_colour(self) -> String {
        ansi_colour_string_helper(self.as_word(), self.ansi_colour())
    }

    const fn as_letter(self) -> &'static str {
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

    fn as_letter_colour(self) -> String {
        ansi_colour_string_helper(self.as_letter(), self.ansi_colour())
    }

    fn as_coloured_peg(self) -> String {
        ansi_colour_string_helper(PEG_SYMBOL, self.ansi_colour())
    }

    const fn ansi_colour(self) -> usize {
        match self {
            Self::Red => 91,
            Self::Green => 92,
            Self::Yellow => 93,
            Self::Blue => 94,
            Self::Purple => 95,
            Self::Aqua => 96,
            Self::White => 97,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct IncompleteCodeError;
impl Display for IncompleteCodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "incomplete code given")
    }
}
impl Error for IncompleteCodeError {}

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
    fn as_string(self, in_colour: bool) -> String {
        if in_colour {
            self.as_string_colour()
        } else {
            self.as_string_mono()
        }
    }

    fn as_string_mono(self) -> String {
        self.0
            .into_iter()
            .map(Peg::as_letter)
            .collect::<Vec<&str>>()
            .join(" ")
    }

    fn as_string_colour(self) -> String {
        self.0
            .into_iter()
            .map(Peg::as_coloured_peg)
            .collect::<Vec<String>>()
            .join(" ")
    }

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
impl TryFrom<InProgressCode> for Code {
    type Error = IncompleteCodeError;

    fn try_from(value: InProgressCode) -> Result<Self, Self::Error> {
        if !value.is_complete() {
            return Err(IncompleteCodeError);
        }

        let mut code_contents = [Peg::Red; CODE_LEN];
        // Safe to unwrap here- we already checked that there are no `None`s in `value`.
        for (i, peg) in value.0.into_iter().map(Option::<Peg>::unwrap).enumerate() {
            code_contents[i] = peg;
        }
        Ok(Code(code_contents))
    }
}

/// An in-progress guess.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct InProgressCode([Option<Peg>; CODE_LEN]);
impl InProgressCode {
    fn new() -> Self {
        Self([None; CODE_LEN])
    }

    fn is_complete(self) -> bool {
        !self.0.iter().any(Option::<Peg>::is_none)
    }

    fn as_string(self, in_colour: bool) -> String {
        self.0
            .into_iter()
            .map(|maybe_peg| match (maybe_peg, in_colour) {
                (Some(peg), true) => peg.as_coloured_peg(),
                (Some(peg), false) => peg.as_letter().to_string(),
                (None, true) => ansi_colour_string_helper(NO_PEG_SYMBOL, 37),
                (None, false) => NO_PEG_SYMBOL.to_string(),
            })
            .collect::<Vec<String>>()
            .join(" ")
    }

    // Returns `true` if and only if a peg was added.
    fn push_peg(&mut self, peg: Peg) -> bool {
        if let Some(index) = self.current_peg_index() {
            // OK to index- Self::current_peg_index only returns indicies within bounds.
            self.0[index] = Some(peg);
            true
        } else {
            false
        }
    }

    fn pop_peg(&mut self) -> Option<Peg> {
        let pop_index = (match self.current_peg_index() {
            None => CODE_LEN,
            Some(index) => index,
        })
        .saturating_sub(1);
        // OK to index- Self::current_peg_index only returns indicies within bounds, and
        // `saturating_sub` ensures we don't overflow. Finally, CODE_LEN is indeed an out-of-bounds
        // index, but then the `saturating_sub` brings it within bounds (i.e. CODE_LEN - 1).
        self.0[pop_index].take()
    }

    // Returns `None` if the guess is full.
    fn current_peg_index(self) -> Option<usize> {
        let mut index = 0;
        let mut pegs_iter = self.0.iter();
        while let Some(Some(_)) = pegs_iter.next() {
            index += 1;
        }
        if index >= CODE_LEN { None } else { Some(index) }
    }
}

/// A given piece of info.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Hint {
    RightPlace,
    RightColour,
    Neither,
}
impl Hint {
    fn as_string(self, in_colour: bool) -> String {
        if in_colour {
            self.as_string_colour()
        } else {
            self.as_str_mono().to_string()
        }
    }

    fn as_str_mono(self) -> &'static str {
        match self {
            Self::RightPlace => RIGHT_PLACE_SYMBOL,
            Self::RightColour => RIGHT_COLOUR_SYMBOL,
            Self::Neither => NEITHER_SYMBOL,
        }
    }

    fn as_string_colour(self) -> String {
        ansi_colour_string_helper(self.as_str_mono(), self.ansi_colour())
    }

    fn ansi_colour(self) -> usize {
        match self {
            Self::RightPlace => 97,
            Self::RightColour => 37,
            Self::Neither => 90,
        }
    }
}

/// A given sequence of hints. Gives info on how much of the player's guess is correct.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Feedback([Hint; CODE_LEN]);
impl Feedback {
    fn from_compare(actual_code: Code, guess: Code) -> Self {
        let mut num_right_place = 0;
        let mut num_right_colour = 0;

        // Since this loop runs `CODE_LEN` times and can't increment `num_right_place` AND
        // `num_right_colour` at the same time, `num_right_place` + `num_right_colour` <=
        // `CODE_LEN` = `feedback.len()`
        for (i, peg) in actual_code.0.iter().enumerate() {
            if &guess.0[i] == peg {
                num_right_place += 1;
            } else if guess.0.contains(peg) {
                num_right_colour += 1;
            }
        }

        let mut feedback = Self([Hint::Neither; CODE_LEN]);
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

    fn as_string(self, in_colour: bool) -> String {
        self.0
            .into_iter()
            .map(|h| h.as_string(in_colour))
            .collect::<Vec<String>>()
            .join(" ")
    }
}

/// A given [`Code`] and the associated [`Feedback`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Guess {
    code: Code,
    feedback: Feedback,
}
impl Guess {
    fn create(game_state: &GameState, code: Code) -> Self {
        Self {
            code,
            feedback: Feedback::from_compare(game_state.actual_code, code),
        }
    }

    fn empty_guess_string(in_colour: bool) -> String {
        // Use an empty in-progress code to spoof an empty guess
        Self::string_helper(
            &InProgressCode::new().as_string(in_colour),
            &Feedback([Hint::Neither; CODE_LEN]).as_string(in_colour),
        )
    }

    fn as_string(self, in_colour: bool) -> String {
        Self::string_helper(
            &self.code.as_string(in_colour),
            &self.feedback.as_string(in_colour),
        )
    }

    fn string_helper(fmt_1: &str, fmt_2: &str) -> String {
        format!("{} | {}", fmt_1, fmt_2)
    }
}

/// The state of the game.
#[derive(Clone, Debug)]
struct GameState {
    actual_code: Code,
    guesses: [Option<Guess>; NUM_GUESSES],
    in_colour: bool,
    current_guess: InProgressCode,
}
impl GameState {
    fn new(in_colour: bool) -> Self {
        // TODO generate random code
        Self {
            actual_code: Code([Peg::Red, Peg::Green, Peg::Yellow, Peg::Blue, Peg::Purple]),
            guesses: [None; NUM_GUESSES],
            in_colour,
            current_guess: InProgressCode::new(),
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

    // Returns `None` if no more guesses are left.
    fn next_guess_index(&self) -> Option<usize> {
        let mut index = 0;
        let mut guesses_iter = self.guesses.iter();
        while let Some(Some(_)) = guesses_iter.next() {
            index += 1;
        }
        if index >= NUM_GUESSES {
            None
        } else {
            Some(index)
        }
    }

    fn push_guess(&mut self) {
        let Ok(code) = self.current_guess.try_into() else {
            return;
        };
        let Some(index) = self.next_guess_index() else {
            return;
        };

        // OK to index- Self::current_guess_index only returns indicies within bounds.
        self.guesses[index] = Some(Guess::create(self, code));

        self.current_guess = InProgressCode::new();
    }

    fn render(&self) {
        for maybe_guess in self.guesses.iter().rev() {
            if let Some(guess) = maybe_guess {
                println!("{}", guess.as_string(self.in_colour));
            } else {
                println!("{}", Guess::empty_guess_string(self.in_colour));
            }
        }
        println!("---------------------");
        println!("{}", self.current_guess.as_string(self.in_colour));
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

fn render_upper_status(in_colour: bool) {
    println!("{LOGO}");
    println!();
    println!(
        "Guess the secret pattern of {CODE_LEN} unique colours in {NUM_GUESSES} guesses or less."
    );
    println!("Type the first letter of a colour to place a peg.");
    println!("Press <Backspace> to undo. Press <Enter> to confirm your complete guess.");
    println!("Press <Ctrl+Q> to quit.");
    println!(
        "'{}' means a peg is the correct colour and correct position.",
        Hint::RightPlace.as_string(in_colour)
    );
    println!(
        "'{}' means a peg is the correct colour, but in the incorrect position.",
        Hint::RightColour.as_string(in_colour)
    );
    let colours_list = if in_colour {
        &*COLOURS_LIST_COLOUR
    } else {
        &*COLOURS_LIST_MONO
    };
    println!("Possible colours: {colours_list}");
    println!("=====================");
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
    // println!(
    //     "{} | {}",
    //     Peg::list_string(&guess.0, game_state.in_colour),
    //     Hint::list_string(&game_state.get_feedback(guess).0, game_state.in_colour)
    // );
}

fn read_line() -> Result<String, Errno> {
    let console = Console::open()?;
    Ok(String::from_utf8_lossy(&console.read_line(64)?).into_owned())
}

fn set_read_timeouts(min_bytes_read: u8, max_time_passed: u8) -> Result<(), Errno> {
    STDIN.lock().set_control_character(
        SetTermAttrsCmd::Tcsetsf,
        ControlCharIndex::Min,
        min_bytes_read,
    )?;
    STDIN.lock().set_control_character(
        SetTermAttrsCmd::Tcsetsf,
        ControlCharIndex::Time,
        max_time_passed,
    )
}

fn enter_raw_mode() -> Result<(), Errno> {
    STDIN.lock().set_input_mode_flags(
        SetTermAttrsCmd::Tcsetsf,
        InputModeFlags::IGNBRK
            | InputModeFlags::BRKINT
            | InputModeFlags::PARMRK
            | InputModeFlags::ISTRIP
            | InputModeFlags::INLCR
            | InputModeFlags::ICRNL
            | InputModeFlags::IXON,
        false,
    )?;
    STDIN
        .lock()
        .set_output_mode_flags(SetTermAttrsCmd::Tcsetsf, OutputModeFlags::OPOST, false)?;
    STDIN.lock().set_local_mode_flags(
        SetTermAttrsCmd::Tcsetsf,
        LocalModeFlags::ECHO
            | LocalModeFlags::ECHONL
            | LocalModeFlags::ICANON
            | LocalModeFlags::ISIG
            | LocalModeFlags::IEXTEN,
        false,
    )?;
    STDIN
        .lock()
        .set_control_mode_flags(SetTermAttrsCmd::Tcsetsf, ControlModeFlags::CSIZE, true)?;
    Ok(())
}

fn restore_terminal(termios: &Termios) -> Result<(), Errno> {
    STDIN.lock().set_termios(SetTermAttrsCmd::Tcsetsf, termios)
}

fn clear_screen() {
    print!("{CLEAR_SCREEN}{CURSOR_TOP_LEFT}");
}

fn refresh_screen(game_state: &GameState) {
    clear_screen();
    render_upper_status(game_state.in_colour);
    game_state.render();
}

fn poll_input() -> Result<u8, Errno> {
    let mut byte_buf = [0];

    // Try to read a byte from stdin.
    loop {
        match STDIN.lock().read(&mut byte_buf) {
            Ok(0) | Err(Errno::Eagain) => {
                // Nothing was read. Try again.
            }
            Ok(_) => {
                // A byte was read. Return it.
                return Ok(byte_buf[0]);
            }
            Err(e) => {
                // Non-retryable error. Return the error.
                return Err(e);
            }
        }
    }
}

fn main(_args: &[String], _env_vars: &[EnvVar]) -> ExitStatus {
    let orig_termios = try_exit!(STDIN.lock().termios());

    try_exit!(enter_raw_mode());
    try_exit!(set_read_timeouts(READ_MIN_BYTES_READ, READ_MAX_TIME_PASSED));
    clear_screen();

    let mut game_state = GameState::new(true);

    loop {
        refresh_screen(&game_state);
        let input_byte = try_exit!(poll_input());

        match input_byte {
            EXIT_CODE => {
                break;
            }
            ENTER_CODE => {
                game_state.push_guess();
            }
            BACKSP_CODE => {
                game_state.current_guess.pop_peg();
            }
            byte => {
                if let Some(peg) = Peg::try_from_char(byte as char) {
                    game_state.current_guess.push_peg(peg);
                }
            }
        }
    }

    // loop {
    //     refresh_screen(&game_state);
    // }

    // while game_state.remaining_guesses() > 0 {
    //     let guess = try_exit!(read_guess());
    //     response(&game_state, guess);
    //
    //     // Check if win
    //     if guess == game_state.actual_code {
    //         println!(
    //             "You cracked the code in {} guess(es)!",
    //             game_state.current_guess_number()
    //         );
    //         return ExitStatus::ExitSuccess;
    //     }
    //
    //     game_state.guesses[game_state.current_guess_number()] = Some(guess);
    // }

    // Out of guesses. Lost.
    // println!("You ran out of guesses :(");
    // println!(
    //     "Actual code: {}",
    //     Peg::list_string(&game_state.actual_code.0, game_state.in_colour)
    // );

    try_exit!(restore_terminal(&orig_termios));
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
    fn in_progress_code_push() {
        let mut ipc = InProgressCode::new();
        let mut expected = [None; CODE_LEN];
        assert_eq!(ipc.0, expected);

        for i in 0..CODE_LEN {
            assert!(ipc.push_peg(Peg::Red));
            expected[i] = Some(Peg::Red);
            assert_eq!(ipc.0, expected);
        }

        // Ensure pushing past limit is handled correctly
        assert!(!ipc.push_peg(Peg::Blue));
        assert_eq!(ipc.0, expected);
    }

    #[test_case]
    fn in_progress_code_pop() {
        let mut ipc = InProgressCode([Some(Peg::Red); CODE_LEN]);
        let mut expected = [Some(Peg::Red); CODE_LEN];

        for i in 0..CODE_LEN {
            assert_eq!(ipc.pop_peg(), Some(Peg::Red));
            expected[CODE_LEN - (i + 1)] = None;
            assert_eq!(ipc.0, expected);
        }

        // Ensure popping off empty guess is handled correctly
        assert_eq!(ipc.pop_peg(), None);
        assert_eq!(ipc.0, expected);
    }

    #[test_case]
    fn in_progress_code_multicolour_push_pop() {
        let mut ipc = InProgressCode::new();
        let mut expected = [None; CODE_LEN];
        assert_eq!(ipc.0, expected);

        assert!(ipc.push_peg(Peg::Red));
        expected[0] = Some(Peg::Red);
        assert_eq!(ipc.0, expected);

        assert!(ipc.push_peg(Peg::Blue));
        expected[1] = Some(Peg::Blue);
        assert_eq!(ipc.0, expected);

        assert!(ipc.push_peg(Peg::Red));
        expected[2] = Some(Peg::Red);
        assert_eq!(ipc.0, expected);

        assert_eq!(ipc.pop_peg(), Some(Peg::Red));
        expected[2] = None;
        assert_eq!(ipc.0, expected);

        assert_eq!(ipc.pop_peg(), Some(Peg::Blue));
        expected[1] = None;
        assert_eq!(ipc.0, expected);

        assert_eq!(ipc.pop_peg(), Some(Peg::Red));
        expected[0] = None;
        assert_eq!(ipc.0, expected);

        assert_eq!(ipc.pop_peg(), None);
        assert_eq!(ipc.0, expected);
    }

    // #[test_case]
    // fn gs_guess_num_remaining() {
    //     let mut gs = GameState::new(true);
    //     for i in 0..NUM_GUESSES {
    //         assert_eq!(gs.current_guess_number(), i);
    //         assert_eq!(gs.remaining_guesses(), NUM_GUESSES - i);
    //         gs.guesses[i] = Some(Code([Peg::Red; CODE_LEN]));
    //         assert_eq!(gs.current_guess_number(), i + 1);
    //     }
    //     assert_eq!(gs.remaining_guesses(), 0);
    //     assert_eq!(gs.current_guess_number(), NUM_GUESSES);
    // }
    //
    // #[test_case]
    // fn code_from_valid_list() {
    //     assert_eq!(
    //         Code::try_from_str("RED bluE a green W"),
    //         Ok(Code([
    //             Peg::Red,
    //             Peg::Blue,
    //             Peg::Aqua,
    //             Peg::Green,
    //             Peg::White
    //         ])),
    //     );
    // }
    //
    // #[test_case]
    // fn code_from_valid_single_chars() {
    //     assert_eq!(
    //         Code::try_from_str("wpyyg"),
    //         Ok(Code([
    //             Peg::White,
    //             Peg::Purple,
    //             Peg::Yellow,
    //             Peg::Yellow,
    //             Peg::Green,
    //         ]))
    //     );
    // }
    //
    // #[test_case]
    // fn code_from_other_whitespace_list() {
    //     assert_eq!(
    //         Code::try_from_str("r\tblue\t\tgreen YELLOW     aqua"),
    //         Ok(Code([
    //             Peg::Red,
    //             Peg::Blue,
    //             Peg::Green,
    //             Peg::Yellow,
    //             Peg::Aqua,
    //         ]))
    //     );
    // }
    //
    // #[test_case]
    // fn code_from_too_short_list() {
    //     assert_eq!(
    //         Code::try_from_str("red blue green yellow"),
    //         Err(CodeFromStrError::TooFew),
    //     );
    // }
    //
    // #[test_case]
    // fn code_from_too_long_list() {
    //     assert_eq!(
    //         Code::try_from_str("aqua white purple blue yellow green"),
    //         Err(CodeFromStrError::TooMany),
    //     );
    // }
    //
    // #[test_case]
    // fn code_from_incorrect_str_len() {
    //     assert_eq!(Code::try_from_str("rgybpa"), Err(CodeFromStrError::TooFew),);
    // }
    //
    // #[test_case]
    // fn code_from_list_with_commas() {
    //     assert_eq!(
    //         Code::try_from_str("aqua, GREEN, w, b, p"),
    //         Ok(Code([
    //             Peg::Aqua,
    //             Peg::Green,
    //             Peg::White,
    //             Peg::Blue,
    //             Peg::Purple
    //         ]))
    //     );
    // }
}
