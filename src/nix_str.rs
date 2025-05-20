//! Functionality centred around syscall-compatible strings.

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use core::iter::IntoIterator;

use crate::NULL_BYTE;

/// An owned, null-terminated string of valid UTF-8 bytes intended for use with Linux syscalls.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NixString(Vec<u8>);
impl NixString {
    /// Creates a new, empty [`NixString`].
    #[must_use]
    pub fn null() -> Self {
        Self(Vec::from([NULL_BYTE]))
    }
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
}
impl TryFrom<Vec<u8>> for NixString {
    type Error = core::str::Utf8Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        // Fail if bytes aren't valid UTF-8
        str::from_utf8(&value)?;
        // Filter out all null bytes
        let mut filtered_bytes = value
            .into_iter()
            .filter(|&byte| byte != NULL_BYTE)
            .collect::<Vec<u8>>();
        // Push a null byte to the end
        filtered_bytes.push(NULL_BYTE);

        Ok(Self(filtered_bytes))
    }
}
impl From<String> for NixString {
    fn from(value: String) -> Self {
        // OK to unwrap here; the String type is guaranteed to be valid UTF-8.
        #[allow(clippy::unwrap_used)]
        Self::try_from(value.into_bytes()).unwrap()
    }
}
impl From<&str> for NixString {
    fn from(value: &str) -> Self {
        Self::from(value.to_string())
    }
}
impl TryFrom<&[u8]> for NixString {
    type Error = core::str::Utf8Error;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::try_from(Vec::from(value))
    }
}
impl From<NixString> for String {
    fn from(value: NixString) -> Self {
        // OK to unwrap here; the NixString type guarantees valid UTF-8
        #[allow(clippy::unwrap_used)]
        String::from_utf8(value.0).unwrap()
    }
}
impl<'a> From<&'a NixString> for &'a str {
    fn from(value: &'a NixString) -> Self {
        // OK to unwrap here; the NixString type guarantees valid UTF-8
        #[allow(clippy::unwrap_used)]
        str::from_utf8(value.bytes()).unwrap()
    }
}
impl<'a> From<&'a NixString> for &'a [u8] {
    fn from(value: &'a NixString) -> Self {
        value.bytes()
    }
}

/// Create a [`Vec<NixString>`] from a given vector of types which implement [`Into<NixString>`].
#[must_use]
pub fn vec_into_nix_strings<T: Into<NixString> + Clone>(arr: Vec<T>) -> Vec<NixString> {
    arr.into_iter()
        .map(|elem| <T as core::convert::Into<NixString>>::into(elem))
        .collect()
}

#[cfg(test)]
mod tests;
