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
#![feature(custom_test_frameworks, variant_count)]
#![cfg_attr(test, test_runner(tlenix_core::custom_test_runner))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]

extern crate alloc;

use alloc::{
    fmt::Display,
    string::{String, ToString},
    vec::Vec,
};
use core::{error::Error, mem::variant_count, panic::PanicInfo, write};

use lazy_static::lazy_static;
use tlenix_core::{
    EnvVar, Errno, eprintln, format, parse_argv_envp, print, println,
    process::{self, ExitStatus},
    rand,
    streams::STDIN,
    term::{
        ControlCharIndex, ControlModeFlags, InputModeFlags, LocalModeFlags, OutputModeFlags,
        SetTermAttrsCmd, Termios,
    },
    try_exit,
};

const LOGO: &str = r" _______  _______  ______   _______  _______  ______    _______  _______  ___   _  _______  ______   
|       ||       ||      | |       ||  _    ||    _ |  |       ||   _   ||   | | ||       ||    _ |  
|       ||   _   ||  _    ||    ___|| |_|   ||   | ||  |    ___||  |_|  ||   |_| ||    ___||   | ||  
|       ||  | |  || | |   ||   |___ |       ||   |_||_ |   |___ |       ||      _||   |___ |   |_||_ 
|      _||  |_|  || |_|   ||    ___||  _   | |    __  ||    ___||       ||     |_ |    ___||    __  |
|     |_ |       ||       ||   |___ | |_|   ||   |  | ||   |___ |   _   ||    _  ||   |___ |   |  | |
|_______||_______||______| |_______||_______||___|  |_||_______||__| |__||___| |_||_______||___|  |_|";

const HEADER_LINES: usize = 16;
const GAME_START_LINE: usize = HEADER_LINES + 1;

const PANIC_TITLE: &str = "codebreaker";

const CODE_LEN: usize = 5;
const NUM_GUESSES: usize = 8;

const PEG_SYMBOL: &str = "o";
const NO_PEG_SYMBOL: &str = ".";
const RIGHT_PLACE_SYMBOL: &str = "!";
const RIGHT_COLOUR_SYMBOL: &str = "?";
const NEITHER_SYMBOL: &str = ".";

const CLEAR_SCREEN: &str = "\u{001b}[2J";
const CURSOR_TOP_LEFT: &str = "\u{001b}[H";
const HIDE_CURSOR: &str = "\u{001b}[?25l";
const SHOW_CURSOR: &str = "\u{001b}[?25h";
const ENTER_ALT_SCREEN: &str = "\u{001b}[?1049h";
const LEAVE_ALT_SCREEN: &str = "\u{001b}[?1049l";
const CLEAR_LINE: &str = "\u{001b}[2K";

const READ_MIN_BYTES_READ: u8 = 0;
/// In deciseconds
const READ_MAX_TIME_PASSED: u8 = 1;

// CONTROLS
const EXIT_CODE: u8 = ctrl_key(b'q');
const RESTART_CODE: u8 = ctrl_key(b'r');
const ENTER_CODE: u8 = 0x0d;
const BACKSP_CODE: u8 = 0x7f;
const ESC_CODE: u8 = 0x1b;

/// Highest multiple of number of colours under 0xff
const RAND_COLOUR_LIMIT: usize =
    variant_count::<Peg>() * ((u8::MAX as usize) / variant_count::<Peg>());

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
    fn random() -> Self {
        loop {
            let rand_val = rand::get_random_byte(rand::GetRandomFlags::default()).unwrap() as usize;
            // Avoid modulo bias via rejection resampling
            if (rand_val) < RAND_COLOUR_LIMIT {
                let colour_list = Self::iterator().collect::<Vec<Peg>>();
                // OK to index here- we are moduloing by the number of enum variants
                return colour_list[rand_val % variant_count::<Peg>()];
            }
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

/// A given code. A sequence of [`Peg`]s.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Code([Peg; CODE_LEN]);
impl Code {
    fn random() -> Self {
        let mut contents = [Peg::Red; CODE_LEN];
        let mut chosen_pegs = Vec::with_capacity(CODE_LEN);
        for peg in &mut contents {
            // Must resample until unique colour is generated.
            'resample_rand_peg: loop {
                let rand_peg = Peg::random();
                if !chosen_pegs.contains(&rand_peg) {
                    chosen_pegs.push(rand_peg);
                    *peg = rand_peg;
                    break 'resample_rand_peg;
                }
            }
        }

        Self(contents)
    }

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
        Self {
            actual_code: Code::random(),
            guesses: [None; NUM_GUESSES],
            in_colour,
            current_guess: InProgressCode::new(),
        }
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

    fn is_won(&self) -> bool {
        self.guesses
            .iter()
            .any(|maybe_guess| maybe_guess.map(|guess| guess.code) == Some(self.actual_code))
    }

    fn is_lost(&self) -> bool {
        !self.is_won() && !self.guesses.iter().any(&Option::<Guess>::is_none)
    }

    fn render(&self) {
        // Move to start of game area
        print!("\u{001b}[{GAME_START_LINE};1H");
        for maybe_guess in self.guesses.iter().rev() {
            if let Some(guess) = maybe_guess {
                println!("{}{}", CLEAR_LINE, guess.as_string(self.in_colour));
            } else {
                println!(
                    "{}{}",
                    CLEAR_LINE,
                    Guess::empty_guess_string(self.in_colour)
                );
            }
        }
        println!("{}---------------------", CLEAR_LINE);
        println!(
            "{}{}",
            CLEAR_LINE,
            self.current_guess.as_string(self.in_colour)
        );
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
    println!("Press <Ctrl+R> to restart, or <Ctrl+Q> to quit.");
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

fn terminal_setup() -> Result<(), Errno> {
    print!("{ENTER_ALT_SCREEN}{HIDE_CURSOR}");
    enter_raw_mode()?;
    set_read_timeouts(READ_MIN_BYTES_READ, READ_MAX_TIME_PASSED)?;
    clear_screen();
    Ok(())
}

fn restore_terminal(termios: &Termios) -> Result<(), Errno> {
    STDIN
        .lock()
        .set_termios(SetTermAttrsCmd::Tcsetsf, termios)?;
    print!("{LEAVE_ALT_SCREEN}{SHOW_CURSOR}");
    Ok(())
}

fn clear_screen() {
    print!("{CLEAR_SCREEN}{CURSOR_TOP_LEFT}");
}

fn refresh_screen(game_state: &GameState) {
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
    try_exit!(terminal_setup());

    'game: loop {
        clear_screen();
        let mut game_state = GameState::new(true);
        render_upper_status(game_state.in_colour);

        loop {
            // Redraw game
            refresh_screen(&game_state);

            // Check if game has been won or lost
            if game_state.is_won() {
                println!(
                    "\nCONGRATULATIONS! You guessed the code in {} guess(es)!",
                    game_state.next_guess_index().unwrap_or(NUM_GUESSES)
                );
            } else if game_state.is_lost() {
                println!("\nSadly, you failed to guess the code...");
                println!(
                    "The code was {}",
                    game_state.actual_code.as_string(game_state.in_colour)
                );
            }

            if game_state.is_won() || game_state.is_lost() {
                println!("Type <Esc> to exit, type <Enter> to play again...");
                loop {
                    match try_exit!(poll_input()) {
                        ESC_CODE | EXIT_CODE => {
                            break 'game;
                        }
                        ENTER_CODE | RESTART_CODE => {
                            continue 'game;
                        }
                        _ => {}
                    }
                }
            }

            // Handle input
            match try_exit!(poll_input()) {
                EXIT_CODE => {
                    break 'game;
                }
                ENTER_CODE => {
                    game_state.push_guess();
                }
                RESTART_CODE => {
                    continue 'game;
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
    }

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
}
