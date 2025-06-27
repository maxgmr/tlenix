//! The [`termios`](https://www.man7.org/linux/man-pages/man3/termios.3.html) data structure, which
//! provides a general terminal interface.

const TERMIOS_CC_SIZE: usize = 32;
const TERMIOS2_CC_SIZE: usize = 19;

/// A general terminal interface derived from the
/// [`termios`](https://www.man7.org/linux/man-pages/man3/termios.3.html) type.
#[derive(Clone, Debug, PartialEq)]
struct Termios {
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
impl From<TermiosRaw> for Termios {
    fn from(value: TermiosRaw) -> Self {
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
}
impl From<Termios2Raw> for Termios {
    fn from(value: Termios2Raw) -> Self {
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
}

/// A raw terminal data type received from calls to
/// [`ioctl`](https://man7.org/linux/man-pages/man2/ioctl.2.html).
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
struct TermiosRaw {
    iflag: u32,
    oflag: u32,
    cflag: u32,
    lflag: u32,
    line: u8,
    cc: [u8; TERMIOS_CC_SIZE],
    ispeed: u32,
    ospeed: u32,
}

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

bitflags::bitflags! {
    /// All the different input mode flags within the
    /// [`termios`](https://www.man7.org/linux/man-pages/man3/termios.3.html) data structure.
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct InputModeFlags: u32 {
        /// Ignore BREAK condition on input.
        const IGNBRK = 0x0000_0001;
        /// If `IGNBRK` is set, a BREAK is ignored. If `IGNBRK` _isn't_ set, but `BRKINT` _is_ set,
        /// then a BREAK causes the input and output queues to be flushed, and if the terminal is
        /// the controlling terminal of a foreground process group, it will cause a `SIGINT` to be
        /// sent to this foreground process group. When neither `IGNBRK` nor `BRKINT` are set, a
        /// BREAK reads as a null byte, except when `PARMRK`, in which case it reads as the
        /// sequence `\377 \0 \0`.
        const BRKINT = 0x0000_0002;
        /// Ignore framing errors and parity errors.
        const IGNPAR = 0x0000_0004;
        /// If this bit is set, input bytes with parity/framing errors are marked when passed into
        /// the program. Only meaningful when `INPCK` is set and `IGNPAR` isn't.If neither `IGNPAR`
        /// nor `PARMRK` are set, read a character with a parity/framing error as null.
        const PARMRK = 0x0000_0008;
        /// Enable input parity checking.
        const INPCK = 0x0000_0010;
        /// Strip off eighth bit.
        const ISTRIP = 0x0000_0020;
        /// Translate NL to CR on input.
        const INLCR = 0x0000_0040;
        /// Ignore CR on input.
        const IGNCR = 0x0000_0080;
        /// Translate CR to NL on input (unless `IGNCR` is set).
        const ICRNL = 0x0000_0100;
        /// Enable XON/XOFF flow control on output.
        const IXON = 0x0000_0400;
        /// Typing any character will restart stopped output.
        const IXANY = 0x0000_0800;
        /// Enable XON/XOFF flow control on output.
        const IXOFF = 0x0000_1000;
        /// Input is UTF-8. Allows character-erase to be correctly performed in cooked mode.
        const IUTF8 = 0x0000_4000;
    }
}

bitflags::bitflags! {
    /// All the different output mode flags within the
    /// [`termios`](https://www.man7.org/linux/man-pages/man3/termios.3.html) data structure.
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct OutputModeFlags: u32 {
        /// Enable implementation-defined output processing.
        const OPOST = 0x0000_0001;
        /// Map NL to CR-NL on output.
        const ONLCR = 0x0000_0004;
        /// Map CR to LN on output.
        const OCRNL = 0x0000_0008;
        /// Don't output CR at column 0.
        const ONOCR = 0x0000_0010;
        /// NL is assumed to perform the CR function; the kernel's idea of the current column is
        /// set to 0 after both NL and CR.
        const ONLRET = 0x0000_0020;
        /// Send fill characters for a delay, rather than using a timed delay.
        const OFILL = 0x0000_0040;
        /// Newline delay mask. Values are `NL0` and `NL1`.
        const NLDLY = 0x0000_0100;
        /// Carriage return delay mask. Values are `CR0`, `CR1`, `CR2`, `CR3`.
        const CRDLY = 0x0000_0600;
        /// Horizontal tab delay mask. Values are `TAB0`, `TAB1`, `TAB2`, `TAB3`.
        const TABDLY = 0x0000_1800;
        /// Vertical tab delay mask. Values are `VT0` or `VT1`.
        const VTDLY = 0x0000_4000;
        /// Form feed delay mask. Values are `FF0` or `FF1`.
        const FFDLY = 0x0000_8000;
    }
}

bitflags::bitflags! {
    /// All the different control mode flags.
    /// [`termios`](https://www.man7.org/linux/man-pages/man3/termios.3.html) data structure.
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ControlModeFlags: u32 {
        /// Baud speed mask (4+1 bits).
        const CBAUD = 0x0000_100F;
        /// Extra baud speed mask (1 bit).
        const CBAUDEX = 0x0000_1000;
        /// Character size mask. Values are `CS5`, `CS6`, `CS7`, or `CS8`.
        const CSIZE = 0x0000_0030;
        /// Set two stop bits instead of one.
        const CSTOPB = 0x0000_0040;
        /// Enable receiver.
        const CREAD = 0x0000_0080;
        /// Enable parity generation on output and parity checking for input.
        const PARENB = 0x0000_0100;
        /// If set, then parity for input and output is odd; otherwise, even parity is used.
        const PARODD = 0x0000_0200;
        /// Lower modem control lines after last process closes the device.
        const HUPCL = 0x0000_0400;
        /// Ignore modem control lines.
        const CLOCAL = 0x0000_0800;
        /// Mask for input speeds. The values are the same as `CBAUD`, shifted left `IBSHIFT` bits.
        const CIBAUD = 0x100F_0000;
        /// Use "stick" (mark/space) parity: if `PARODD` is set, the parity bit is always 1;
        /// otherwise, it's always 0.
        const CMSPAR = 0x4000_0000;
        /// Enable RTS/CTS (hardware) flow control.
        const CRTSCTS = 0x8000_0000;
    }
}

bitflags::bitflags! {
    /// All the different local mode flags within the
    /// [`termios`](https://www.man7.org/linux/man-pages/man3/termios.3.html) data structure.
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct LocalModeFlags: u32 {
        /// When any of the characters INTR, QUIT, SUSP, or DSUSP are received, generate the
        /// corresponding signal.
        const ISIG = 0x0000_0001;
        /// Enable canonical mode.
        const ICANON = 0x0000_0002;
        /// Echo input characters.
        const ECHO = 0x0000_0008;
        /// If `ICANON` is also set, the ERASE chracter erases the preceding input character, and
        /// WERASE erases the preceding word.
        const ECHOE = 0x0000_0010;
        /// If `ICANON` is also set, the KILL character erases the current line.
        const ECHOK = 0x0000_0020;
        /// If `ICANON` is also set, echo the NL character even if ECHO is not set.
        const ECHONL = 0x0000_0040;
        /// If `ECHO` is set, terminal special characters other than TAB, NL, START, and STOP are
        /// echoed as `^X`, where X is the character with ASCII code 0x40 creater than the special
        /// character (caret notation).
        const ECHOCTL = 0x0000_0200;
        /// If `ICANON` and `ECHO` are also set, characters are printed as they are being created.
        const ECHOPRT = 0x0000_0400;
        /// If `ICANON` is also set, KILL is echoed by erasing each character on the line, as
        /// specified by `ECHOE` and `ECHOPRT`.
        const ECHOKE = 0x0000_0800;
        /// Disable flushing the input and output queues when generating signals for the INT, QUIT,
        /// and SUSP characters.
        const NOFLSH = 0x0000_0080;
        /// Send the `SIGTTOU` signal to the process group of a background process which tries to
        /// write to its controlling terminal.
        const TOSTOP = 0x0000_0100;
        /// Enable implementation-defined input processing.
        const IEXTEN = 0x0000_8000;
    }
}
