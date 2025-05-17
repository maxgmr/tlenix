//! Functionality centred around syscall-compatible strings.

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use core::iter::IntoIterator;

const NULL_BYTE: u8 = b'\0';

/// An owned, null-terminated string of valid UTF-8 bytes intended for use with Linux syscalls.
pub struct NixString(Vec<u8>);
impl NixString {
    /// Returns a raw pointer to the [`NixString`]'s buffer.
    #[must_use]
    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.0.as_ptr()
    }

    /// Returns the byte slice of the [`NixString`].
    #[must_use]
    #[inline]
    pub fn bytes(&self) -> &[u8] {
        &self.0
    }

    /// Returns a [`&str`] referencing the bytes of this [`NixString`].
    pub fn try_as_str(&self) -> Result<&str, core::str::Utf8Error> {
        str::from_utf8(&self.0)
    }
}
impl From<Vec<u8>> for NixString {
    fn from(value: Vec<u8>) -> Self {
        crate::println!("{:?}", value);
        // Filter out all null bytes
        let mut filtered_bytes = value
            .into_iter()
            .filter(|&byte| byte != NULL_BYTE)
            .collect::<Vec<u8>>();
        crate::println!("{:?}", filtered_bytes);
        // Push a null byte to the end
        filtered_bytes.push(NULL_BYTE);
        crate::println!("{:?}", filtered_bytes);

        Self(filtered_bytes)
    }
}
impl From<String> for NixString {
    fn from(value: String) -> Self {
        Self::from(value.into_bytes())
    }
}
impl From<&str> for NixString {
    fn from(value: &str) -> Self {
        Self::from(value.to_string())
    }
}
impl From<&[u8]> for NixString {
    fn from(value: &[u8]) -> Self {
        Self::from(Vec::from(value))
    }
}
impl TryFrom<NixString> for String {
    type Error = alloc::string::FromUtf8Error;

    fn try_from(value: NixString) -> Result<Self, Self::Error> {
        String::from_utf8(value.0)
    }
}

#[cfg(test)]
mod tests;
