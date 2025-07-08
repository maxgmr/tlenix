//! Functionality related to the
//! [`ioctl`](https://www.man7.org/linux/man-pages/man2/ioctl.2.html) Linux system call.

use super::termios::{Termios2Raw, TermiosRaw};
use super::winsize::WinSizeRaw;
use crate::{
    Errno, SyscallNum,
    fs::FileDescriptor,
    syscall_result,
    term::{Termios, WinSize},
};

const TCGETS: u64 = 0x0000_5401;
// const TCGETS2: u64 = 0x802c_542a;
const TIOCGWINSZ: u64 = 0x0000_5413;

/// The different commands for setting terminal attributes.
#[repr(u64)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SetTermAttrsCmd {
    /// Set the current serial port settings to the given `termios`.
    ///
    /// This command applies the changes immediately, regardless of pending I/O.
    Tcsets = 0x0000_5402,
    /// Allow the output buffer to drain, and set the current serial port settings to the given
    /// `termios`.
    ///
    /// This command waits for all output to be transmitted before applying changes.
    Tcsetsw = 0x0000_5403,
    /// Allow the output buffer to drain, discard pending input, and set the current serial port
    /// settings to the given `termios`.
    ///
    /// This command waits for output to complete *and* discards unread input before applying
    /// changes.
    Tcsetsf = 0x0000_5404,
    /// Set the current serial port settings to the given `termios2`.
    ///
    /// This command applies the changes immediately, regardless of pending I/O.
    Tcsets2 = 0x402c_542b,
    /// Allow the output buffer to drain, and set the current serial port settings to the given
    /// `termios2`.
    ///
    /// This command waits for all output to be transmitted before applying changes.
    Tcsetsw2 = 0x402c_542c,
    /// Allow the output buffer to drain, discard pending input, and set the current serial port
    /// settings to the given `termios2`.
    ///
    /// This command waits for output to complete *and* discards unread input before applying
    /// changes.
    Tcsetsf2 = 0x402c_542d,
}
impl SetTermAttrsCmd {
    /// Returns `true` if the command uses a `termios2`; returns `false` otherwise.
    fn uses_termios2(self) -> bool {
        matches!(self, Self::Tcsets2 | Self::Tcsetsw2 | Self::Tcsetsf2)
    }
}

/// Gets the attributes of the given [`FileDescriptor`].
///
/// Internally uses the
/// [`ioctl`](https://www.man7.org/linux/man-pages/man2/ioctl.2.html) Linux system call.
///
/// # Errors
///
/// This function propagates any [`Errno`]s incurred during the underling `ioctl` syscall, most
/// notably [`Errno::Eperm`] in the case of insufficient permissions, or [`Errno::Enotty`] if the
/// given file descriptor doesn't refer to a TTY.
pub(crate) fn get_term_attrs(file_descriptor: FileDescriptor) -> Result<Termios, Errno> {
    let mut termios_raw = TermiosRaw::default();
    // SAFETY: The number and the type of the parameters matches the system call definition. The
    // `TCGETS` value is a valid `cmd`. The `TermiosRaw` struct is the right size and alignment to
    // serve as the provided buffer.
    unsafe {
        syscall_result!(
            SyscallNum::Ioctl,
            file_descriptor,
            TCGETS,
            &raw mut termios_raw as usize
        )?;
    }
    Ok(termios_raw.into())
}

/// Sets the attributes of the given [`FileDescriptor`] to the given [`Termios`].
///
/// Uses the given [`SetTermAttrsCmd`] to determine specific behaviour.
///
/// Internally uses the
/// [`ioctl`](https://www.man7.org/linux/man-pages/man2/ioctl.2.html) Linux system call.
///
/// # Errors
///
/// This function propagates any [`Errno`]s incurred during the underling `ioctl` syscall, most
/// notably [`Errno::Eperm`] in the case of insufficient permissions, or [`Errno::Enotty`] if the
/// given file descriptor doesn't refer to a TTY.
#[allow(clippy::similar_names)]
pub(crate) fn set_term_attrs(
    cmd: SetTermAttrsCmd,
    file_descriptor: FileDescriptor,
    termios: &Termios,
) -> Result<(), Errno> {
    // Define the raw types to ensure they live long enough
    let termios_raw: TermiosRaw = termios.into();
    let termios2_raw: Termios2Raw = termios.into();

    let termios_ptr = if cmd.uses_termios2() {
        &raw const termios2_raw as usize
    } else {
        &raw const termios_raw as usize
    };

    // SAFETY: The number and the type of the parameters matches the system call definition. The
    // `SetTermAttrsCmd` enum restricts `cmd` values to those which write the given `Termios` to
    // the given file descriptor.
    // `TCSETS` value is a valid `cmd`. The pointer either points to `TermiosRaw` or `Termios2Raw`.
    // Both structs match the expected size and alignment, and the termios vs termios2 decision is
    // made based on the `SetTermAttrsCmd` enum. Finally, defining `termios_raw` and `termios2_raw` at the function-level scope guarantees the raw pointer is valid throughout the syscall.
    unsafe {
        syscall_result!(SyscallNum::Ioctl, file_descriptor, cmd as u64, termios_ptr)?;
    }
    Ok(())
}

/// Get the size of the terminal pointed to by the given [`FileDescriptor`].
///
/// # Errors
///
/// This function propagates any [`Errno`]s incurred by the underlying call to `ioctl`.
pub(crate) fn get_term_size(file_descriptor: FileDescriptor) -> Result<WinSize, Errno> {
    let mut win_size_raw: WinSizeRaw = WinSizeRaw::default();

    // SAFETY: The number and type of the parameters matches the system call definition. The use of
    // the `TIOCGWINSZ` constant for the `cmd` arg ensures *only* that command is possible. The
    // `WinSizeRaw` type is the correct alignment and size to store the returned WinSize.
    // `win_size_raw` lives long enough for the raw pointer to be valid.
    unsafe {
        syscall_result!(
            SyscallNum::Ioctl,
            file_descriptor,
            TIOCGWINSZ,
            &raw mut win_size_raw as usize
        )?;
    }

    Ok(win_size_raw.into())
}
