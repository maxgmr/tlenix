//! A collection of ANSI escape sequences used for terminal control.
//!
//! See [`console_codes(4)`](https://www.man7.org/linux/man-pages/man4/console_codes.4.html) for a
//! reference of supported codes.

extern crate alloc;

use alloc::string::{String, ToString};

macro_rules! define_ecma_single_consts {
    [$(
        $(#[$doc:meta])+
        $name:ident = $num:literal;
    )*] => {
        $(
            $(#[$doc])+
            pub const $name: &str = concat!("\u{001b}[", $num, "m");
        )*
    }
}
define_ecma_single_consts![
    /// Reset all graphic attributes to their defaults.
    ANSI_RESET_GRAPHIC = "";
    /// Set bold text.
    ANSI_BOLD = "1";
    /// Set half-bright text.
    ANSI_HALF_BRIGHT = "2";
    /// Set italic text.
    ANSI_ITALIC = "3";
    /// Set blinking text.
    ANSI_BLINK = "5";
    /// Invert text colours.
    ANSI_INVERT = "7";
    /// Set underline.
    ANSI_UNDERLINE = "21";
    /// Set normal intensity.
    ANSI_NORMAL_INTENSITY = "22";
    /// Turn off italics.
    ANSI_ITALIC_OFF = "23";
    /// Turn off underline.
    ANSI_UNDERLINE_OFF = "24";
    /// Turn off blinking.
    ANSI_BLINK_OFF = "25";
    /// Turn off inverted text colours.
    ANSI_INVERT_OFF = "27";
    /// Set the foreground colour to black.
    ANSI_FG_BLACK = "30";
    /// Set the foreground colour to red.
    ANSI_FG_RED = "31";
    /// Set the foreground colour to green.
    ANSI_FG_GREEN = "32";
    /// Set the foreground colour to yellow.
    ANSI_FG_YELLOW = "33";
    /// Set the foreground colour to blue.
    ANSI_FG_BLUE = "34";
    /// Set the foreground colour to magenta.
    ANSI_FG_MAGENTA = "35";
    /// Set the foreground colour to cyan.
    ANSI_FG_CYAN = "36";
    /// Set the foreground colour to white.
    ANSI_FG_WHITE = "37";
    /// Set the default foreground colour.
    ANSI_FG_DEFAULT = "39";
    /// Set the background colour to black.
    ANSI_BG_BLACK = "40";
    /// Set the background colour to red.
    ANSI_BG_RED = "41";
    /// Set the background colour to green.
    ANSI_BG_GREEN = "42";
    /// Set the background colour to yellow.
    ANSI_BG_YELLOW = "43";
    /// Set the background colour to blue.
    ANSI_BG_BLUE = "44";
    /// Set the background colour to magenta.
    ANSI_BG_MAGENTA = "45";
    /// Set the background colour to cyan.
    ANSI_BG_CYAN = "46";
    /// Set the background colour to white.
    ANSI_BG_WHITE = "47";
    /// Set the default background colour.
    ANSI_BG_DEFAULT = "49";
    /// Set the foreground colour to bright black.
    ANSI_FG_B_BLACK = "90";
    /// Set the foreground colour to bright red.
    ANSI_FG_B_RED = "91";
    /// Set the foreground colour to bright green.
    ANSI_FG_B_GREEN = "92";
    /// Set the foreground colour to bright yellow.
    ANSI_FG_B_YELLOW = "93";
    /// Set the foreground colour to bright blue.
    ANSI_FG_B_BLUE = "94";
    /// Set the foreground colour to bright magenta.
    ANSI_FG_B_MAGENTA = "95";
    /// Set the foreground colour to bright cyan.
    ANSI_FG_B_CYAN = "96";
    /// Set the foreground colour to bright white.
    ANSI_FG_B_WHITE = "97";
];

macro_rules! define_ansi_cursors {
    [$(
        $(#[$doc:meta])+
        $name:ident = $suffix:literal;
    )*] => {
        $(
            $(#[$doc])+
            #[macro_export]
            macro_rules! $name {
                () => {
                    concat!("\u{001b}[", $suffix)
                };
                ($n:literal) => {
                    concat!("\u{001b}[", $n, $suffix)
                }
            }
        )*
    };
}

define_ansi_cursors![
    /// Move the cursor up the given number of lines.
    ansi_cursor_up = "A";
    /// Move the cursor down the given number of lines.
    ansi_cursor_down = "B";
    /// Move the cursor right the given number of lines.
    ansi_cursor_right = "C";
    /// Move the cursor left the given number of lines.
    ansi_cursor_left = "D";
    /// Move the cursor down the given number of rows, to column 1.
    ansi_cursor_down_col1 = "E";
    /// Move the cursor up the given number of rows, to column 1.
    ansi_cursor_up_col1 = "F";
    /// Move the cursor to the given column in the current row.
    ansi_cursor_to_col = "G";
    /// Move the cursor to the given row in the current column.
    ansi_cursor_to_row = "d";
];

/// Move the cursor to a specified row and column.
#[macro_export]
macro_rules! ansi_cursor_to_pos {
    ($row:literal, $col:literal) => {
        concat!("\u{001b}[", $row, ";", $col, "H")
    };
}

/// Move the cursor to the top left of the display.
pub const ANSI_CURSOR_TOP_LEFT: &str = "\u{001b}[H";

/// Hide the cursor.
pub const ANSI_HIDE_CURSOR: &str = "\u{001b}[?25l";
/// Show the cursor.
pub const ANSI_SHOW_CURSOR: &str = "\u{001b}[?25h";

/// Get the cursor position.
pub const ANSI_GET_CURSOR_POS: &str = "\u{001b}[6n";

/// Erase from the cursor to the end of the display.
pub const ANSI_ERASE_CURSOR_TO_END: &str = "\u{001b}[J";
/// Erase from the start of the display to the cursor.
pub const ANSI_ERASE_START_TO_CURSOR: &str = "\u{001b}[1J";
/// Erase the whole display.
pub const ANSI_ERASE_DISPLAY: &str = "\u{001b}[2J";
/// Erase the whole display, along with the scroll-back buffer.
pub const ANSI_ERASE_DISPLAY_SCROLLBACK: &str = "\u{001b}[3J";

/// Erase from the cursor to the end of the line.
pub const ANSI_ERASE_REMAINING_LINE: &str = "\u{001b}[K";
/// Erase from the start of the line to the cursor.
pub const ANSI_ERASE_LINE_TO_CURSOR: &str = "\u{001b}[1K";
/// Erase the entire line.
pub const ANSI_ERASE_LINE: &str = "\u{001b}[2K";

/// Clear all keyboard LEDs.
pub const ANSI_CLEAR_KB_LEDS: &str = "\u{001b}[0q";
/// Set the scroll lock keyboard LED.
pub const ANSI_SCROLL_LOCK_LED: &str = "\u{001b}[1q";
/// Set the num lock keyboard LED.
pub const ANSI_NUM_LOCK_LED: &str = "\u{001b}[2q";
/// Set the caps lock keyboard LED.
pub const ANSI_CAPS_LOCK_LED: &str = "\u{001b}[3q";

/// Save the current cursor location.
pub const ANSI_SAVE_CURSOR: &str = "\u{001b}[s";
/// Restore the saved cursor location.
pub const ANSI_RESTORE_CURSOR: &str = "\u{001b}[u";

/// An ECMA-48 display attribute.
#[allow(missing_docs)]
#[repr(usize)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum DisplayAttr {
    Reset = 0,
    Bold = 1,
    HalfBright = 2,
    Italic = 3,
    Underscore = 4,
    Blink = 5,
    Inverted = 7,
    Underline = 21,
    NormalIntensity = 22,
    ItalicOff = 23,
    UnderlineOff = 24,
    BlinkOff = 25,
    InvertedOff = 27,
    BlackFg = 30,
    RedFg = 31,
    GreenFg = 32,
    YellowFg = 33,
    BlueFg = 34,
    MagentaFg = 35,
    CyanFg = 36,
    WhiteFg = 37,
    DefaultFg = 39,
    BlackBg = 40,
    RedBg = 41,
    GreenBg = 42,
    YellowBg = 43,
    BlueBg = 44,
    MagentaBg = 45,
    CyanBg = 46,
    WhiteBg = 47,
    DefaultBg = 49,
    BrightBlackFg = 90,
    BrightRedFg = 91,
    BrightGreenFg = 92,
    BrightYellowFg = 93,
    BrightBlueFg = 94,
    BrightMagentaFg = 95,
    BrightCyanFg = 96,
    BrightWhiteFg = 97,
    BrightBlackBg = 100,
    BrightRedBg = 101,
    BrightGreenBg = 102,
    BrightYellowBg = 103,
    BrightBlueBg = 104,
    BrightMagentaBg = 105,
    BrightCyanBg = 106,
    BrightWhiteBg = 107,
}

/// Creates an ECMA-48 sequence capable of setting the given [`DisplayAttr`]s.
#[must_use]
pub fn display_seq(display_attrs: &[DisplayAttr]) -> String {
    let mut seq = String::with_capacity((display_attrs.len() * 4) + 3);
    seq.push_str("\u{001b}[");
    for (i, &display_attr) in display_attrs.iter().enumerate() {
        seq.push_str((display_attr as usize).to_string().as_str());
        if i < (display_attrs.len() - 1) {
            seq.push(';');
        }
    }
    seq.push('m');
    seq
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn empty_ecma_display_seq() {
        assert_eq!(display_seq(&[]).as_str(), "\u{001b}[m");
    }

    #[test_case]
    fn ecma_display_seq() {
        assert_eq!(
            display_seq(&[
                DisplayAttr::Bold,
                DisplayAttr::BlueBg,
                DisplayAttr::RedFg,
                DisplayAttr::BlinkOff
            ])
            .as_str(),
            "\u{001b}[1;44;31;25m"
        );
    }
}
