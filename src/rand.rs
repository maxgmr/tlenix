//! Module containing functionality related to randomness.

use crate::{Errno, SyscallNum, syscall_result};

bitflags::bitflags! {
    /// The options which can be passed to the [`crate::rand::get_random_bytes`] function.
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct GetRandomFlags: u32 {
        /// If the entropy pool hasn't been initialized or no random bytes are available,
        /// immediately return [`Errno::Eagain`].
        const NONBLOCK = 1;
        /// Use `/dev/random` instead of `/dev/urandom`, which is limited by environmental noise.
        /// If the available bytes can't fill the given buffer, then the call returns just the
        /// available bytes.
        const RANDOM = 2;
    }
}
impl Default for GetRandomFlags {
    fn default() -> Self {
        Self::empty()
    }
}

/// Returns a random byte, i.e., a random number from 0 to 255 (inclusive).
///
/// Essentially a wrapper around the
/// [`getrandom`](https://man7.org/linux/man-pages/man2/getrandom.2.html) Linux system call.
///
/// # Errors
///
/// This function propagates any [`Errno`]s incurred by the underlying call to `getrandom`.
pub fn get_random_byte(flags: GetRandomFlags) -> Result<u8, Errno> {
    let mut buf = [0];
    match get_random_bytes(&mut buf, flags) {
        // OK to index, the buffer is explicitly initialized with one value above.
        Ok(1) => Ok(buf[0]),
        Ok(_) => Err(Errno::Eagain),
        Err(errno) => Err(errno),
    }
}

/// Fills the provided buffer with random bytes, returning the number of bytes read.
///
/// Essentially a wrapper around the
/// [`getrandom`](https://man7.org/linux/man-pages/man2/getrandom.2.html) Linux system call.
///
/// # Errors
///
/// This function propagates any [`Errno`]s incurred by the underlying call to `getrandom`.
pub fn get_random_bytes(buffer: &mut [u8], flags: GetRandomFlags) -> Result<usize, Errno> {
    let buf_ptr = buffer.as_mut_ptr();

    // SAFETY: `SyscallNum` restricts the first arg to a valid value. `buffer` isn't dropped before
    // `buf_ptr`, so `buf_ptr` is valid. The `count` syscall arg is calculated at runtime directly
    // from the length of `buffer`, so it's guaranteed to be correct. The `GetRandomFlags` type
    // prevents invalid flags from being sent to the syscall.
    unsafe { syscall_result!(SyscallNum::Getrandom, buf_ptr, buffer.len(), flags.bits()) }
}
