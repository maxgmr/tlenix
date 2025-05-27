#![allow(clippy::unwrap_used)]

use super::*;
use crate::assert_err;

const TEST_STR: &str = "Hello, world!";
const TEST_NULL_TERM: &str = "Hello, world!\0";
const TEST_EMPTY: &str = "";
const TEST_BYTES: [u8; 13] = *b"Hello, world!";
const TEST_NON_UTF8: [u8; 2] = [0xC0, 0x80];

#[test_case]
fn nbytes_from_string() {
    let string = TEST_STR.to_string();
    let expected_bytes = TEST_NULL_TERM.as_bytes();
    let nbytes: NixBytes = string.into();
    assert_eq!(nbytes.bytes(), expected_bytes);
}

#[test_case]
fn nbytes_from_byte_vec() {
    let bytes = Vec::from(TEST_STR.as_bytes());
    let expected_bytes = TEST_NULL_TERM.as_bytes();
    let nbytes: NixBytes = bytes.into();
    assert_eq!(nbytes.bytes(), expected_bytes);
}

#[test_case]
fn nbytes_from_byte_slice() {
    let nbytes = NixBytes::from(&TEST_BYTES[..]);
    assert_eq!(nbytes.bytes(), TEST_NULL_TERM.as_bytes());
}

#[test_case]
fn nbytes_already_null_term() {
    let nbytes = NixBytes::from(TEST_NULL_TERM);
    assert_eq!(nbytes.bytes(), TEST_NULL_TERM.as_bytes());
}

#[test_case]
fn null_nbytes() {
    let expected_bytes = *b"\0";
    let nbytes = NixBytes::null();
    assert_eq!(nbytes.bytes(), expected_bytes);
}

#[test_case]
fn nbytes_bytes() {
    let nbytes = NixBytes::from(TEST_STR);
    assert_eq!(nbytes.bytes(), TEST_NULL_TERM.as_bytes());
}

#[test_case]
fn nbytes_to_str() {
    let nbytes = NixBytes::from(TEST_STR);
    assert_eq!(str::from_utf8(nbytes.bytes()).unwrap(), TEST_NULL_TERM);
}

#[test_case]
fn nbytes_from_empty() {
    let nbytes = NixBytes::from(TEST_EMPTY);
    assert_eq!(nbytes.bytes(), [b'\0']);
}

#[test_case]
fn nbytes_non_utf8() {
    let nbytes = NixBytes::from(&TEST_NON_UTF8[..]);
    let mut expected: Vec<u8> = Vec::from(TEST_NON_UTF8);
    expected.push(NULL_BYTE);
    assert_eq!(nbytes.bytes(), expected);
}

#[test_case]
fn nbytes_non_utf8_str_fails() {
    let nbytes = NixBytes::from(&TEST_NON_UTF8[..]);
    assert_err!(str::from_utf8(nbytes.bytes()), core::str::Utf8Error { .. });
}

#[test_case]
fn null_nbytes_as_str() {
    let nbytes = NixBytes::null();
    let my_str: &str = (&nbytes).try_into().unwrap();
    assert_eq!(my_str, "\0");
}

#[test_case]
fn null_nbytes_as_string() {
    let nbytes = NixBytes::null();
    let test_string = String::try_from(nbytes).unwrap();
    assert_eq!(&test_string, "\0");
}
