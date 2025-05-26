//! Different types related to process management.

use num_enum::TryFromPrimitive;

use crate::{
    Errno,
    ipc::{SigInfoRaw, Signo},
};

/// All the possible values which can be returned by a process.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ExitStatus {
    /// The process exited successfully. Corresponds to '0' in C.
    ExitSuccess,
    /// The process encountered some failure and did not exit successfully.
    ExitFailure(i32),
    /// The process was terminated by a signal.
    Terminated(Signo),
    /// The process was stopped by a signal.
    Stopped(Signo),
}
impl From<ExitStatus> for i32 {
    fn from(value: ExitStatus) -> Self {
        #[allow(clippy::enum_glob_use)]
        use ExitStatus::*;

        match value {
            ExitSuccess => 0,
            ExitFailure(val) => val,
            Terminated(signo) | Stopped(signo) => signo as i32,
        }
    }
}
impl core::fmt::Display for ExitStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        #[allow(clippy::enum_glob_use)]
        use ExitStatus::*;

        match self {
            ExitSuccess => write!(f, "exited successfully"),
            ExitFailure(code) => write!(f, "exited with code {code}"),
            Terminated(signo) => write!(f, "killed by signal {signo}"),
            Stopped(signo) => write!(f, "stopped by signal {signo}"),
        }
    }
}

#[repr(i32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, TryFromPrimitive)]
pub enum ChildCode {
    /// Child process called `_exit(2)`.
    Exited = 1,
    /// Child process killed by signal.
    Killed = 2,
    /// Child process killed by signal and dumped core.
    Dumped = 3,
    /// Traced child process has trapped.
    Trapped = 4,
    /// Child process stopped by signal.
    Stopped = 5,
    /// Child process continued by [`Signo::SigCont`].
    Continued = 6,
}

/// Detailed information returned by
/// [`wait`](https://man7.org/linux/man-pages/man2/waitid.2.html) system calls.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WaitInfo {
    /// Process ID of the child.
    pub child_pid: i32,
    /// The real user ID of the child.
    pub child_uid: u32,
    /// Either the exit status of the child or the signal that caused the child to
    /// terminate/stop/continue.
    pub status: i32,
    /// The reason the wait returned.
    pub child_code: ChildCode,
}
impl WaitInfo {
    /// Try to interpret [`Self::status`] based on [`Self::child_code`] as a [`Signo`].
    ///
    /// If [`Self::child_code`] states the wait returned because of a signal, then the function
    /// will return the [`Signo`] that cause the wait to return. Otherwise, the function will
    /// return [`None`].
    #[must_use]
    pub fn try_interpret_signal(&self) -> Option<Signo> {
        #[allow(clippy::enum_glob_use)]
        use ChildCode::*;

        match self.child_code {
            Killed | Dumped | Stopped | Continued => self.status.try_into().ok(),
            _ => None,
        }
    }
}
impl TryFrom<WaitInfo> for ExitStatus {
    type Error = Errno;
    fn try_from(value: WaitInfo) -> Result<Self, Self::Error> {
        #[allow(clippy::enum_glob_use)]
        use ChildCode::*;

        match (value.child_code, value.status) {
            (Killed | Dumped, s) => Ok(Self::Terminated(s.try_into().map_err(|_| Errno::Einval)?)),
            (Stopped, s) => Ok(Self::Stopped(s.try_into().map_err(|_| Errno::Einval)?)),
            (_, 0) => Ok(Self::ExitSuccess),
            (_, s) => Ok(Self::ExitFailure(s)),
        }
    }
}
impl TryFrom<SigInfoRaw> for WaitInfo {
    type Error = Errno;

    // We only care about the raw bits themselves, not how Rust interprets them
    #[allow(clippy::cast_sign_loss)]
    fn try_from(value: SigInfoRaw) -> Result<Self, Self::Error> {
        Ok(Self {
            child_pid: value.pid,
            child_uid: value.uid,
            status: value.status,
            child_code: value.code.try_into().map_err(|_| Errno::Einval)?,
        })
    }
}

bitflags::bitflags! {
    /// All the different option flags which can be passed to [`crate::process::wait`]. Each set
    /// flag defines a possible state change to wait for.
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct WaitOptions: i32 {
        /// Return immediately if no child has exited.
        const WNOHANG = 0x1;
        /// Wait for children which have been stopped by the delivery of a signal.
        const WSTOPPED = 0x2;
        /// Wait for children which have terminated.
        const WEXITED = 0x4;
        /// Wait for (previously stopped) children which have been resumed by
        /// [`crate::ipc::Signo::SigCont`].
        const WCONTINUED = 0x8;
        /// Leave the child in a waitable state; a later wait call can be used again to retrieve
        /// the child status information.
        const WNOWAIT = 0x100_0000;
    }
}
impl Default for WaitOptions {
    fn default() -> Self {
        Self::WEXITED
    }
}

/// Denotes which child state changes to wait for.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WaitIdType {
    /// Wait for any child, ignoring the given `id`.
    All = 0,
    /// Wait for the child whose process ID matches the given `id`.
    Pid = 1,
    /// Wait for any child whose process group ID matches the given `id`. If `id` is zero, wait for
    /// any child in the same process group as the caller's process group at the time of the call.
    Pgid = 2,
    /// Wait for the child referred to by the PID file descriptor specified in the given `id`.
    PidFd = 3,
}
