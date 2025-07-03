//! Functionality related to the
//! [`ioctl`](https://www.man7.org/linux/man-pages/man2/ioctl.2.html) Linux system call.

use super::termios::TermiosRaw;
use crate::{Errno, SyscallNum, fs::FileDescriptor, syscall_result, term::Termios};

const TCGETS: u64 = 0x0000_5401;
const TCSETS: u64 = 0x0000_5402;
// const TCSETSW: u64 = 0x0000_5403;
// const TCSETSF: u64 = 0x0000_5404;
// const TCGETS2: u64 = 0x802c_542a;
// const TCSETS2: u64 = 0x402c_542b;
// const TCSETSW2: u64 = 0x402c_542c;
// const TCSETSF2: u64 = 0x402c_542d;

/// Sets the attributes of the given [`FileDescriptor`] to the given [`Termios`].
///
/// Internally uses the
/// [`ioctl`](https://www.man7.org/linux/man-pages/man2/ioctl.2.html) Linux system call.
///
/// # Errors
///
/// This function propagates any [`Errno`]s incurred during the underling `ioctl` syscall, most
/// notably [`Errno::Eperm`] in the case of insufficient permissions, or [`Errno::Enotty`] if the
/// given file descriptor doesn't refer to a TTY.
pub(crate) fn set_term_attrs(
    file_descriptor: FileDescriptor,
    termios: &Termios,
) -> Result<(), Errno> {
    let termios_raw: TermiosRaw = termios.into();
    // SAFETY: The number and the type of the parameters matches the system call definition. The
    // `TCSETS` value is a valid `cmd`. The `TermiosRaw` struct is the right size and alignment to
    // serve as the provided buffer.
    unsafe {
        syscall_result!(
            SyscallNum::Ioctl,
            file_descriptor,
            TCSETS,
            &raw const termios_raw as usize
        )?;
    }
    Ok(())
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
