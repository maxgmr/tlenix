//! Functionality centred around syscall-compatible bytes.

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use crate::NULL_BYTE;

/// An owned, null-terminated string of bytes intended for use with Linux syscalls.
///
/// These bytes are arbitrary and therefore not necessarily valid UTF-8. To guarantee valid UTF-8,
/// use [`crate::NixBytes`] instead.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NixBytes(Vec<u8>);
impl NixBytes {
    /// Creates a new, empty [`NixBytes`].
    #[must_use]
    pub fn null() -> Self {
        Self(Vec::from([NULL_BYTE]))
    }
    /// Returns a raw pointer to the [`NixBytes`]'s buffer.
    #[must_use]
    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.0.as_ptr()
    }

    /// Returns the byte slice of the [`NixBytes`].
    #[must_use]
    #[inline]
    pub fn bytes(&self) -> &[u8] {
        &self.0
    }
}
impl Default for NixBytes {
    fn default() -> Self {
        Self::null()
    }
}
impl From<Vec<u8>> for NixBytes {
    fn from(value: Vec<u8>) -> Self {
        // Filter out all null bytes
        let mut filtered_bytes = value
            .into_iter()
            .filter(|&byte| byte != NULL_BYTE)
            .collect::<Vec<u8>>();
        // Push a null byte to the end
        filtered_bytes.push(NULL_BYTE);

        Self(filtered_bytes)
    }
}
impl From<Vec<i8>> for NixBytes {
    fn from(value: Vec<i8>) -> Self {
        // OK to allow here; we're simply reinterpreting the raw bytes as chars. The UTF-8 check
        // later on catches any issues.
        #[allow(clippy::cast_sign_loss)]
        Self::from(value.into_iter().map(|i| i as u8).collect::<Vec<u8>>())
    }
}
impl From<String> for NixBytes {
    fn from(value: String) -> Self {
        Self::from(value.into_bytes())
    }
}
impl From<&String> for NixBytes {
    fn from(value: &String) -> Self {
        Self::from(value.as_str())
    }
}
impl From<&str> for NixBytes {
    fn from(value: &str) -> Self {
        Self::from(value.to_string())
    }
}
impl From<&[u8]> for NixBytes {
    fn from(value: &[u8]) -> Self {
        Self::from(Vec::from(value))
    }
}
impl From<&[i8]> for NixBytes {
    fn from(value: &[i8]) -> Self {
        Self::from(Vec::from(value))
    }
}
impl TryFrom<NixBytes> for String {
    type Error = alloc::string::FromUtf8Error;
    fn try_from(value: NixBytes) -> Result<Self, Self::Error> {
        String::from_utf8(value.0)
    }
}
impl<'a> TryFrom<&'a NixBytes> for &'a str {
    type Error = core::str::Utf8Error;
    fn try_from(value: &'a NixBytes) -> Result<Self, Self::Error> {
        str::from_utf8(value.bytes())
    }
}
impl<'a> From<&'a NixBytes> for &'a [u8] {
    fn from(value: &'a NixBytes) -> Self {
        value.bytes()
    }
}

#[cfg(test)]
mod tests;
