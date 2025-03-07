//! Functionality related to data types, conversion, and processing.

const TO_NULL_TERM_STR_ERR_MSG: &str = "input is too long for NullTermStr buffer";

/// A null-terminated byte slice of a static length.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NullTermStr<const N: usize>([u8; N]);
impl<const N: usize> NullTermStr<N> {
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
}
