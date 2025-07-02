//! Module containing the [`super::Termios`] control character indicies.

// /// An index corresponding to a particular control character within [`Termios`].
// #[repr(usize)]
// #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
// enum ControlCharIndex {
//     /// End-of-file character (EOF). Causes the pending tty buffer to be sent to the waiting user
//     /// program without waiting for end-of-line. If it is the first character of the line, the
//     /// [`read(2)`](https://www.man7.org/linux/man-pages/man2/read.2.html) in the user program
//     /// returns 0, which signifies end-of-file. Recognized when `ICANON` is set, and then not
//     /// passed as input.
//     Eof = 4,
//     /// Additional end-of-line character (EOL). Recognized when `ICANON` is set.
//     Eol = 11,
//     /// Additional end-of-line character (EOL). Recognized when `ICANON` is set.
//     Eol2 = 16,
//     /// Erase character (ERASE). Erases the previous not-yet-erased character, but does not erase
//     /// past EOF or beginning-of-line. Regonized when `ICANON` is set, and then not passed as
//     /// input.
//     Erase = 2,
//     /// Interrupt character (INTR). Send a [`crate::ipc::Signo::SigInt`] signal. Recognized when
//     /// `ISIG` is set, and then not passed as input.
//     Intr = 0,
//     /// Kill character (KILL). This erases the input since the last EOF or beginning-of-line.
//     /// Recognized when `ICANON` is set, and then not passed as input.
//     Kill = 3,
//     /// Literal next (LNEXT). Quotes the next input character, depriving it of a possible special
//     /// meaning. Recognized when `IEXTEN` is set, and then not passed as input.
//     Lnext = 15,
//     /// Minimum number of characters for noncanonical read (MIN).
//     Min = 6,
//     /// Quit character (QUIT). Send [`crate::ipc::Signo::SigQuit`] signal. Recognized when `ISIG`
//     /// is set, and then not passed as input.
//     Quit = 1,
//     /// Reprint unread characters (REPRINT). Recognized when `ICANON` and `IEXTEN` are set, and
//     /// then not passed as input.
//     Reprint = 12,
//     /// Start character (START). Restarts output stopped by the Stop character. Recognized when
//     /// `IXON` is set, and then not passed as input.
//     Start = 8,
//     /// Stop character (STOP). Stop output until Start character is typed. Recognized when `IXON`
//     /// is set, and then not passed as input.
//     Stop = 9,
//     /// Suspend character (SUSP). Send [`crate::ipc::Signo::SigTstp`] signal. Recognized when
//     /// `ISIG` is set, and then not passed as input.
//     Susp = 10,
// }
