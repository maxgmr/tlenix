//! Functionality related to data types, conversion, and processing.

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

const TO_NULL_TERM_STR_ERR_MSG: &str = "input is too long for NullTermStr buffer";

/// Directly create a [`NullTermStr`] which can be evaluated at compile time.
///
/// The first literal is the literal byte string, and the second literal within the brackets is the
/// length of the [`NullTermStr`] (_not_ the byte string!)
///
/// # Examples
///
/// ```
/// // The byte array itself is 13 bytes long, but with the null terminator it'd be 14.
/// const MY_NULL_TERM_STR: NullTermStr<14> = nulltermstr!(b"Hello, world!");
/// ```
#[macro_export]
macro_rules! nulltermstr {
    ($s:literal) => {
        NullTermStr::__raw_nulltermstr_do_not_use(*concat_bytes!($s, b"\0"))
    };
}

/// A null-terminated [`Vec<u8>`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NullTermString(Vec<u8>);
impl NullTermString {
    /// Returns a raw pointer to the [`NullTermString`]'s buffer.
    #[must_use]
    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.0.as_ptr()
    }

    /// Returns the byte slice of the [`NullTermString`].
    #[must_use]
    #[inline]
    pub fn bytes(&self) -> &[u8] {
        &self.0
    }
}
impl From<String> for NullTermString {
    fn from(value: String) -> Self {
        let mut bytes = value.into_bytes();
        // Ensure null-terminated
        bytes.push(b'\0');
        Self(bytes)
    }
}
impl From<&str> for NullTermString {
    fn from(value: &str) -> Self {
        Self::from(value.to_string())
    }
}
impl From<&[u8]> for NullTermString {
    fn from(value: &[u8]) -> Self {
        let mut bytes = Vec::from(value);
        bytes.push(b'\0');
        Self(bytes)
    }
}

/// A null-terminated byte array of a static length.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NullTermStr<const N: usize>([u8; N]);
impl<const N: usize> NullTermStr<N> {
    /// This should NOT BE USED DIRECTLY! Giving this bytes which _aren't_ null-terminated breaks
    /// _the whole point of this type!_ This function is just for the [`nulltermstr`] macro.
    #[doc(hidden)]
    #[must_use]
    pub const fn __raw_nulltermstr_do_not_use(bytes: [u8; N]) -> Self {
        Self(bytes)
    }

    /// Returns a raw pointer to the [`NullTermStr`]'s buffer.
    #[must_use]
    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.0.as_ptr()
    }

    /// Returns the byte slice of the [`NullTermStr`].
    #[must_use]
    #[inline]
    pub fn bytes(&self) -> &[u8] {
        &self.0
    }
}
impl<const N: usize> TryFrom<&str> for NullTermStr<N> {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if (value.len() + 1) > N {
            // Not enough space!
            return Err(TO_NULL_TERM_STR_ERR_MSG);
        }

        let mut buf = [0x00_u8; N];
        let bytes = value.as_bytes();
        buf[..bytes.len()].copy_from_slice(value.as_bytes());
        // Ensure null terminator is appended
        buf[bytes.len()] = b'\0';

        Ok(NullTermStr(buf))
    }
}
impl<const N: usize> From<[u8; N - 1]> for NullTermStr<N> {
    fn from(value: [u8; N - 1]) -> Self {
        let mut buf = [0x00_u8; N];
        buf[..(N - 1)].copy_from_slice(&value);
        // Ensure null terminator is appended
        buf[N - 1] = b'\0';
        NullTermStr(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_STR: &str = "Hello, world!";
    const TEST_ALREADY_NULL_TERM: &str = "Hello, world!\0";
    const TEST_EMPTY: &str = "";
    const TEST_BYTES: [u8; 13] = *b"Hello, world!";

    #[test_case]
    fn ntstring_from() {
        let expected_bytes = b"Hello, world!\0";
        let my_str = "Hello, world!";
        let my_nts = NullTermString::from(my_str);
        assert_eq!(my_nts.bytes(), expected_bytes);

        let my_string = my_str.to_string();
        let my_nts = NullTermString::from(my_string);
        assert_eq!(my_nts.bytes(), expected_bytes);
    }

    #[test_case]
    fn try_from() {
        let expected = b"Hello, world!\0";
        let result = NullTermStr::<14>::try_from(TEST_STR).unwrap();
        assert_eq!(result.bytes(), expected);
    }

    #[test_case]
    fn too_small() {
        NullTermStr::<13>::try_from(TEST_STR).unwrap_err();
    }

    #[test_case]
    fn lrg_buf() {
        let expected = b"Hello, world!\0\0\0";
        let result = NullTermStr::<16>::try_from(TEST_STR).unwrap();
        assert_eq!(result.bytes(), expected);
    }

    #[test_case]
    fn already_null_term() {
        let expected = b"Hello, world!\0\0";
        let result = NullTermStr::<15>::try_from(TEST_ALREADY_NULL_TERM).unwrap();
        assert_eq!(result.bytes(), expected);
    }

    #[test_case]
    fn empty() {
        NullTermStr::<1>::try_from(TEST_EMPTY).unwrap();
    }

    #[test_case]
    fn from() {
        let expected = *b"Hello, world!\0";
        let result = NullTermStr::<14>::from(TEST_BYTES);
        assert_eq!(result.bytes(), expected);
    }

    #[test_case]
    fn nts_macro() {
        let expected = *b"Hello, world!\0";
        let result = nulltermstr!(b"Hello, world!");
        assert_eq!(expected, result.bytes());
    }
}
