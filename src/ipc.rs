//! Functionality related to inter-process communication.

use core::fmt::Display;

use num_enum::TryFromPrimitive;

/// The raw signal info obtained directly from the kernel.
///
/// See [`sigaction(2)`](https://www.man7.org/linux/man-pages/man2/sigaction.2.html) for more
/// information.
#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct SigInfoRaw {
    /// Signal number
    pub signo: i32,
    /// Errno value
    pub errno: u32,
    /// Signal code
    pub code: i32,
    /// Trap number that caused hardware-generated signal
    pub trapno: i32,
    /// Sending process ID
    pub pid: i32,
    /// Real user ID of sending process
    pub uid: u32,
    /// Exit value or signal
    pub status: i32,
    // We don't really care about the other stuff...
    #[doc(hidden)]
    pub _pad: [i32; 24],
    #[doc(hidden)]
    pub _align: [u64; 0],
}

/// The number of a specific IPC signal.
/// [`signal(7)`](https://www.man7.org/linux/man-pages/man7/signal.7.html) provides more info.
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(i32)]
#[non_exhaustive]
pub enum Signo {
    /// Controlling terminal hangup
    SigHup = 1,
    /// Keyboard interrupt
    SigInt = 2,
    /// Quit from keyboard
    SigQuit = 3,
    /// Illegal instruction
    SigIll = 4,
    /// Trace/breakpoint trap
    SigTrap = 5,
    /// Abort
    SigAbrt = 6,
    /// Bus error (bad memory access)
    SigBus = 7,
    /// Erroneous arithmetic operation
    SigFpe = 8,
    /// Kill signal
    SigKill = 9,
    /// User-defined signal 1
    SigUsr1 = 10,
    /// Invalid memory reference
    SigSegv = 11,
    /// User-defined signal 2
    SigUsr2 = 12,
    /// Broken pipe (write to pipe with no readers)
    SigPipe = 13,
    /// Timer signal
    SigAlrm = 14,
    /// Termination signal
    SigTerm = 15,
    /// Stack fault on coprocessor
    SigStkflt = 16,
    /// Child stopped or terminated
    SigChld = 17,
    /// Continue if stopped
    SigCont = 18,
    /// Stop process
    SigStop = 19,
    /// Stop typed at terminal
    SigTstp = 20,
    /// Background process terminal input
    SigTtin = 21,
    /// Background process terminal output
    SigTtou = 22,
    /// Urgent socket condition
    SigUrg = 23,
    /// CPU time limit exceeded
    SigXcpu = 24,
    /// File size limit exceeded
    SigXfsz = 25,
    /// Virtual alarm clock
    SigVtalrm = 26,
    /// Profiling timer expired
    SigProf = 27,
    /// Window resize signal
    SigWinch = 28,
    /// I/O now possible
    SigIo = 29,
    /// Power failure
    SigPwr = 30,
    /// Bad system call
    SigSys = 31,
}
impl Display for Signo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        #[allow(clippy::enum_glob_use)]
        use Signo::*;
        let s = match self {
            SigHup => "SIGHUP",
            SigInt => "SIGINT",
            SigQuit => "SIGQUIT",
            SigIll => "SIGILL",
            SigTrap => "SIGTRAP",
            SigAbrt => "SIGABRT",
            SigBus => "SIGBUS",
            SigFpe => "SIGFPE",
            SigKill => "SIGKILL",
            SigUsr1 => "SIGUSR1",
            SigSegv => "SIGSEGV",
            SigUsr2 => "SIGUSR2",
            SigPipe => "SIGPIPE",
            SigAlrm => "SIGALRM",
            SigTerm => "SIGTERM",
            SigStkflt => "SIGSTKFLT",
            SigChld => "SIGCHLD",
            SigCont => "SIGCONT",
            SigStop => "SIGSTOP",
            SigTstp => "SIGTSTP",
            SigTtin => "SIGTTIN",
            SigTtou => "SIGTTOU",
            SigUrg => "SIGURG",
            SigXcpu => "SIGXCPU",
            SigXfsz => "SIGXFSZ",
            SigVtalrm => "SIGVTALRM",
            SigProf => "SIGPROF",
            SigWinch => "SIGWINCH",
            SigIo => "SIGIO",
            SigPwr => "SIGPWR",
            SigSys => "SIGSYS",
        };
        write!(f, "{s}")
    }
}
