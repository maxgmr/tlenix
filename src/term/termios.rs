//! The [`termios`](https://www.man7.org/linux/man-pages/man3/termios.3.html) data structure, which
//! provides a general terminal interface.

mod control_char_index;
mod line_discipline;
mod mode_flags;

// RE-EXPORTS

use line_discipline::LineDiscipline;
pub use mode_flags::{ControlModeFlags, InputModeFlags, LocalModeFlags, OutputModeFlags};

const TERMIOS_CC_SIZE: usize = 32;
const TERMIOS2_CC_SIZE: usize = 19;

/// Macro to implement the getter and setter [`Termios`] methods for all the mode flags.
macro_rules! impl_termios_flags_methods {
    [$($flags_t:ty),* $(,)?] => {
        $(paste::paste! {
            /// Gets the value of the given
            #[doc = concat!("[`", stringify!($flags_t), "`]")]
            /// flag.
            ///
            /// If multiple flags are given, then this function will only return `true` if *all*
            /// the given flags are set.
            #[must_use]
            pub(crate) fn [<get_ $flags_t:snake>](&self, flag: $flags_t) -> bool {
                self.[<$flags_t:snake>].contains(flag)
            }

            /// Sets the value of the given
            #[doc = concat!("[`", stringify!($flags_t), "`]")]
            /// flag to the given boolean value.
            ///
            /// If multiple flags are given, then *all* given flags will be set to the given
            /// boolean value.
            pub(crate) fn [<set_ $flags_t:snake>](&mut self, flag: $flags_t, value: bool) {
                self.[<$flags_t:snake>].set(flag, value)
            }
        })*
    };
}

/// A general terminal interface derived from the
/// [`termios`](https://www.man7.org/linux/man-pages/man3/termios.3.html) type.
#[derive(Clone, Debug, PartialEq)]
pub struct Termios {
    input_mode_flags: InputModeFlags,
    output_mode_flags: OutputModeFlags,
    local_mode_flags: LocalModeFlags,
    control_mode_flags: ControlModeFlags,
    line_discipline: LineDiscipline,
    control_characters: [u8; TERMIOS_CC_SIZE],
    input_baud_rate: u32,
    output_baud_rate: u32,
}
impl Termios {
    impl_termios_flags_methods![
        InputModeFlags,
        OutputModeFlags,
        LocalModeFlags,
        ControlModeFlags
    ];

    #[allow(clippy::similar_names)]
    #[allow(clippy::too_many_arguments)]
    fn from_raw_helper(
        iflag: u32,
        oflag: u32,
        lflag: u32,
        cflag: u32,
        line: u8,
        control_characters: [u8; TERMIOS_CC_SIZE],
        ispeed: u32,
        ospeed: u32,
    ) -> Self {
        Self {
            input_mode_flags: InputModeFlags::from_bits_truncate(iflag),
            output_mode_flags: OutputModeFlags::from_bits_truncate(oflag),
            local_mode_flags: LocalModeFlags::from_bits_truncate(lflag),
            control_mode_flags: ControlModeFlags::from_bits_truncate(cflag),
            line_discipline: LineDiscipline::from(line),
            control_characters,
            input_baud_rate: ispeed,
            output_baud_rate: ospeed,
        }
    }
}
macro_rules! impl_from_termiosraw {
    ($($t:ty),+) => {
        $(impl From<$t> for Termios {
            fn from(value: $t) -> Self {
                Self::from_raw_helper(
                    value.iflag,
                    value.oflag,
                    value.lflag,
                    value.cflag,
                    value.line,
                    value.cc,
                    value.ispeed,
                    value.ospeed,
                )
            }
        })+
    };
}
impl_from_termiosraw!(TermiosRaw, &TermiosRaw);
macro_rules! impl_from_termios2raw {
    ($($t:ty),+) => {
        $(impl From<$t> for Termios {
            fn from(value: $t) -> Self {
                let mut control_characters = [0; TERMIOS_CC_SIZE];
                control_characters[..TERMIOS2_CC_SIZE].copy_from_slice(&value.cc);
                Self::from_raw_helper(
                    value.iflag,
                    value.oflag,
                    value.lflag,
                    value.cflag,
                    value.line,
                    control_characters,
                    value.ispeed,
                    value.ospeed,
                )
            }
        })+
    };
}
impl_from_termios2raw!(Termios2Raw, &Termios2Raw);

/// A raw terminal data type received from calls to
/// [`ioctl`](https://man7.org/linux/man-pages/man2/ioctl.2.html).
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TermiosRaw {
    iflag: u32,
    oflag: u32,
    cflag: u32,
    lflag: u32,
    line: u8,
    cc: [u8; TERMIOS_CC_SIZE],
    ispeed: u32,
    ospeed: u32,
}
macro_rules! impl_from_termios_termiosraw {
    ($($t:ty),+) => {
       $(impl From<$t> for TermiosRaw {
           fn from(value: $t) -> Self {
                Self {
                    iflag: value.input_mode_flags.bits(),
                    oflag: value.output_mode_flags.bits(),
                    cflag: value.control_mode_flags.bits(),
                    lflag: value.local_mode_flags.bits(),
                    line: value.line_discipline as u8,
                    cc: value.control_characters,
                    ispeed: value.input_baud_rate,
                    ospeed: value.output_baud_rate,
                }
           }
       })+
    };
}
impl_from_termios_termiosraw!(Termios, &Termios);

/// A raw terminal data type received from calls to
/// [`ioctl`](https://man7.org/linux/man-pages/man2/ioctl.2.html) when used with the '2' versions
/// of different `ioctl` commands.
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
struct Termios2Raw {
    iflag: u32,
    oflag: u32,
    cflag: u32,
    lflag: u32,
    line: u8,
    cc: [u8; TERMIOS2_CC_SIZE],
    ispeed: u32,
    ospeed: u32,
}
macro_rules! impl_from_termios_termios2raw {
    ($($t:ty),+) => {
       $(impl From<$t> for Termios2Raw {
           fn from(value: $t) -> Self {
               Self {
                   iflag: value.input_mode_flags.bits(),
                   oflag: value.output_mode_flags.bits(),
                   cflag: value.control_mode_flags.bits(),
                   lflag: value.local_mode_flags.bits(),
                   line: value.line_discipline as u8,
                   // OK to unwrap here- `TERMIOS2_CC_SIZE` is the static size of `cc`.
                   #[allow(clippy::unwrap_used)]
                   cc: value.control_characters[..TERMIOS2_CC_SIZE]
                       .try_into()
                       .unwrap(),
                   ispeed: value.input_baud_rate,
                   ospeed: value.output_baud_rate,
               }
           }
       })+
    };
}
impl_from_termios_termios2raw!(Termios, &Termios);

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_TERMIOS: Termios = Termios {
        input_mode_flags: InputModeFlags::empty(),
        output_mode_flags: OutputModeFlags::empty(),
        control_mode_flags: ControlModeFlags::empty(),
        local_mode_flags: LocalModeFlags::empty(),
        line_discipline: LineDiscipline::Tty,
        control_characters: [0; TERMIOS_CC_SIZE],
        input_baud_rate: 0,
        output_baud_rate: 0,
    };

    #[test_case]
    fn get_set_termios_input_flags() {
        let mut t = TEST_TERMIOS.clone();

        assert!(!t.get_input_mode_flags(InputModeFlags::IGNBRK));
        t.set_input_mode_flags(InputModeFlags::IGNBRK, true);
        assert!(t.get_input_mode_flags(InputModeFlags::IGNBRK));
        t.set_input_mode_flags(InputModeFlags::IGNBRK, true);
        assert!(t.get_input_mode_flags(InputModeFlags::IGNBRK));
        t.set_input_mode_flags(InputModeFlags::IGNBRK, false);
        assert!(!t.get_input_mode_flags(InputModeFlags::IGNBRK));
        t.set_input_mode_flags(InputModeFlags::IGNBRK, false);
        assert!(!t.get_input_mode_flags(InputModeFlags::IGNBRK));
    }

    #[test_case]
    fn get_set_termios_output_flags() {
        let mut t = TEST_TERMIOS.clone();

        assert!(!t.get_output_mode_flags(OutputModeFlags::ONOCR));
        t.set_output_mode_flags(OutputModeFlags::ONOCR, true);
        assert!(t.get_output_mode_flags(OutputModeFlags::ONOCR));
    }

    #[test_case]
    fn get_set_termios_local_flags() {
        let mut t = TEST_TERMIOS.clone();

        assert!(!t.get_local_mode_flags(LocalModeFlags::ECHO));
        t.set_local_mode_flags(LocalModeFlags::ECHO, true);
        assert!(t.get_local_mode_flags(LocalModeFlags::ECHO));
    }

    #[test_case]
    fn get_set_termios_control_flags() {
        let mut t = TEST_TERMIOS.clone();

        assert!(!t.get_control_mode_flags(ControlModeFlags::PARENB));
        t.set_control_mode_flags(ControlModeFlags::PARENB, true);
        assert!(t.get_control_mode_flags(ControlModeFlags::PARENB));
    }

    #[test_case]
    fn termios_flags_contains() {
        let mut t = TEST_TERMIOS.clone();

        t.set_input_mode_flags(InputModeFlags::IGNBRK, true);
        assert!(!t.get_input_mode_flags(InputModeFlags::IGNPAR | InputModeFlags::IGNBRK));
        t.set_input_mode_flags(InputModeFlags::IGNPAR, true);
        assert!(t.get_input_mode_flags(InputModeFlags::IGNPAR | InputModeFlags::IGNBRK));
    }

    #[test_case]
    fn termios_flags_set_multi() {
        let mut t = TEST_TERMIOS.clone();

        t.set_input_mode_flags(
            InputModeFlags::IGNPAR | InputModeFlags::IGNBRK | InputModeFlags::IGNCR,
            true,
        );
        assert!(t.get_input_mode_flags(
            InputModeFlags::IGNPAR | InputModeFlags::IGNBRK | InputModeFlags::IGNCR
        ));
        t.set_input_mode_flags(InputModeFlags::IGNCR | InputModeFlags::IGNPAR, false);
        assert!(t.get_input_mode_flags(InputModeFlags::IGNBRK));
        assert!(!t.get_input_mode_flags(InputModeFlags::IGNCR));
        assert!(!t.get_input_mode_flags(InputModeFlags::IGNPAR));
    }
}
