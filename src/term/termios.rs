//! The [`termios`](https://www.man7.org/linux/man-pages/man3/termios.3.html) data structure, which
//! provides a general terminal interface.

mod mode_flags;

// RE-EXPORTS

pub use mode_flags::{ControlModeFlags, InputModeFlags, LocalModeFlags, OutputModeFlags};

const TERMIOS_CC_SIZE: usize = 32;
const TERMIOS2_CC_SIZE: usize = 19;

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

/// The terminal line discipline.
#[repr(u8)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum LineDiscipline {
    /// Traditional tty driver
    #[default]
    Tty = 0,
    /// Serial line IP
    Slip = 1,
    /// Bus mouse
    Mouse = 2,
    /// Point-to-point protocol
    Ppp = 3,
    /// Starmode Radio IP
    Strip = 4,
    /// AX.25 packet radio protocol
    Ax25 = 5,
    /// X.25 async
    X25 = 6,
    /// 6pack protocol for serial lines
    SixPack = 7,
    /// Mobitex module
    Masc = 8,
    /// Simatic R3964 module
    R3964 = 9,
    /// Profibus
    ProfibusFdl = 10,
    /// Linux Ir-Da
    Irda = 11,
    /// SMS block mode
    Smsblock = 12,
    /// Sync HDLC
    Hdlc = 13,
    /// Sync PPP
    SyncPpp = 14,
    /// Bluetooth HCI UART
    Hci = 15,
    /// Siemens Gigaset M101 serial DECT adapter
    GigasetM101 = 16,
    /// Serial / USB serial CAN adaptors
    Slcan = 17,
    /// Pulse per second
    Pps = 18,
    /// Codec control over voice modem
    V253 = 19,
    /// CAIF control over voice modem
    Caif = 20,
    /// GSM 0710 Mux
    Gsm0710 = 21,
    /// TI's WL BT/FM/GPS combo chips
    TiWl = 22,
    /// Trace data routing for MIPI P1149.7
    Tracesink = 23,
    /// Trace data routing for MIPI P1149.7
    Tracerouter = 24,
    /// NFC NCI UART
    Nci = 25,
    /// Speakup communication with synths
    Speakup = 26,
    /// Null ldisc used for error handling
    Null = 27,
    /// MCTP-over-serial
    Mctp = 28,
    /// Manual out-of-tree testing
    Development = 29,
    /// ELM327 based OBD-II interfaces
    Can327 = 30,
    /// Always the newest line discipline + 1
    Ldiscs = 31,
}
impl From<u8> for LineDiscipline {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Slip,
            2 => Self::Mouse,
            3 => Self::Ppp,
            4 => Self::Strip,
            5 => Self::Ax25,
            6 => Self::X25,
            7 => Self::SixPack,
            8 => Self::Masc,
            9 => Self::R3964,
            10 => Self::ProfibusFdl,
            11 => Self::Irda,
            12 => Self::Smsblock,
            13 => Self::Hdlc,
            14 => Self::SyncPpp,
            15 => Self::Hci,
            16 => Self::GigasetM101,
            17 => Self::Slcan,
            18 => Self::Pps,
            19 => Self::V253,
            20 => Self::Caif,
            21 => Self::Gsm0710,
            22 => Self::TiWl,
            23 => Self::Tracesink,
            24 => Self::Tracerouter,
            25 => Self::Nci,
            26 => Self::Speakup,
            27 => Self::Null,
            28 => Self::Mctp,
            29 => Self::Development,
            30 => Self::Can327,
            31 => Self::Ldiscs,
            _ => Self::Tty,
        }
    }
}

/// An index corresponding to a particular control character within [`Termios`].
#[repr(usize)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ControlCharIndex {
    /// End-of-file character (EOF). Causes the pending tty buffer to be sent to the waiting user
    /// program without waiting for end-of-line. If it is the first character of the line, the
    /// [`read(2)`](https://www.man7.org/linux/man-pages/man2/read.2.html) in the user program
    /// returns 0, which signifies end-of-file. Recognized when `ICANON` is set, and then not
    /// passed as input.
    Eof = 4,
    /// Additional end-of-line character (EOL). Recognized when `ICANON` is set.
    Eol = 11,
    /// Additional end-of-line character (EOL). Recognized when `ICANON` is set.
    Eol2 = 16,
    /// Erase character (ERASE). Erases the previous not-yet-erased character, but does not erase
    /// past EOF or beginning-of-line. Regonized when `ICANON` is set, and then not passed as
    /// input.
    Erase = 2,
    /// Interrupt character (INTR). Send a [`crate::ipc::Signo::SigInt`] signal. Recognized when
    /// `ISIG` is set, and then not passed as input.
    Intr = 0,
    /// Kill character (KILL). This erases the input since the last EOF or beginning-of-line.
    /// Recognized when `ICANON` is set, and then not passed as input.
    Kill = 3,
    /// Literal next (LNEXT). Quotes the next input character, depriving it of a possible special
    /// meaning. Recognized when `IEXTEN` is set, and then not passed as input.
    Lnext = 15,
    /// Minimum number of characters for noncanonical read (MIN).
    Min = 6,
    /// Quit character (QUIT). Send [`crate::ipc::Signo::SigQuit`] signal. Recognized when `ISIG`
    /// is set, and then not passed as input.
    Quit = 1,
    /// Reprint unread characters (REPRINT). Recognized when `ICANON` and `IEXTEN` are set, and
    /// then not passed as input.
    Reprint = 12,
    /// Start character (START). Restarts output stopped by the Stop character. Recognized when
    /// `IXON` is set, and then not passed as input.
    Start = 8,
    /// Stop character (STOP). Stop output until Start character is typed. Recognized when `IXON`
    /// is set, and then not passed as input.
    Stop = 9,
    /// Suspend character (SUSP). Send [`crate::ipc::Signo::SigTstp`] signal. Recognized when
    /// `ISIG` is set, and then not passed as input.
    Susp = 10,
}
