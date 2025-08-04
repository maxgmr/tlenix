//! Module defining all the different [`super::Termios`] line disciplines.

/// The terminal line discipline.
#[repr(u8)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LineDiscipline {
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
