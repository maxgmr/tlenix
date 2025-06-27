//! The [`termios`](https://www.man7.org/linux/man-pages/man3/termios.3.html) type, which provides
//! a general terminal interface.

/// A general terminal interface derived from the
/// [`termios`](https://www.man7.org/linux/man-pages/man3/termios.3.html) type.
#[derive(Clone, Debug, PartialEq)]
struct Termios {}

/// A raw terminal interface received directory from a call to
/// [`ioctl`](https://man7.org/linux/man-pages/man2/ioctl.2.html).
#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
struct TermiosRaw {}

bitflags::bitflags! {
    /// All the different input mode flags.
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct InputModeFlags: u32 {
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
    /// All the different output mode flags.
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct OutputModeFlags: u32 {
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
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct ControlModeFlags: u32 {
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
    /// All the different local mode flags.
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct LocalModeFlags: u32 {
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
