#![allow(clippy::unwrap_used)]

use super::*;
use crate::assert_err;

const TEST_STR: &str = "Hello, world!";
const TEST_NULL_TERM: &str = "Hello, world!\0";
const TEST_EMPTY: &str = "";
const TEST_BYTES: [u8; 13] = *b"Hello, world!";
const INVALID_UTF8: [u8; 2] = [0xC0, 0x80];

#[test_case]
fn nstring_from_string() {
    let string = TEST_STR.to_string();
    let expected_bytes = TEST_NULL_TERM.as_bytes();
    let my_nstring: NixString = string.into();
    assert_eq!(my_nstring.bytes(), expected_bytes);
}

#[test_case]
fn nstring_from_byte_vec() {
    let bytes = Vec::from(TEST_STR.as_bytes());
    let expected_bytes = TEST_NULL_TERM.as_bytes();
    let my_nstring: NixString = bytes.try_into().unwrap();
    assert_eq!(my_nstring.bytes(), expected_bytes);
}

#[test_case]
fn nstring_from_byte_slice() {
    let my_nstring = NixString::try_from(&TEST_BYTES[..]).unwrap();
    assert_eq!(my_nstring.bytes(), TEST_NULL_TERM.as_bytes());
}

#[test_case]
fn nstring_already_null_term() {
    let my_nstring = NixString::from(TEST_NULL_TERM);
    assert_eq!(my_nstring.bytes(), TEST_NULL_TERM.as_bytes());
}

#[test_case]
fn null_nstring() {
    let expected_bytes = *b"\0";
    let my_nstring = NixString::null();
    assert_eq!(my_nstring.bytes(), expected_bytes);
}

#[test_case]
fn nstring_bytes() {
    let my_nstring = NixString::from(TEST_STR);
    assert_eq!(my_nstring.bytes(), TEST_NULL_TERM.as_bytes());
}

#[test_case]
fn nstring_to_str() {
    let my_nstring = NixString::from(TEST_STR);
    assert_eq!(str::from_utf8(my_nstring.bytes()).unwrap(), TEST_NULL_TERM);
}

#[test_case]
fn nstring_from_empty() {
    let my_nstring = NixString::from(TEST_EMPTY);
    assert_eq!(my_nstring.bytes(), [b'\0']);
}

#[test_case]
fn nstring_invalid_utf8_slice() {
    assert_err!(
        NixString::try_from(&INVALID_UTF8[..]),
        core::str::Utf8Error { .. }
    );
}

#[test_case]
fn nstring_invalid_utf8_vec() {
    assert_err!(
        NixString::try_from(INVALID_UTF8.to_vec()),
        core::str::Utf8Error { .. }
    );
}

#[test_case]
fn null_nstring_as_str() {
    let my_nstring = NixString::null();
    let my_str: &str = (&my_nstring).into();
    assert_eq!(my_str, "");
}

#[test_case]
fn null_nstring_as_string() {
    let my_nstring = NixString::null();
    let test_string = String::from(my_nstring);
    assert_eq!(&test_string, "");
}

#[test_case]
fn nstring_trim_extra_null() {
    const TEST_BYTES: [u8; 3] = [0x4d, NULL_BYTE, NULL_BYTE];
    let nstring = NixString::try_from(&TEST_BYTES[..]).unwrap();
    assert_eq!(nstring.bytes(), &TEST_BYTES[..TEST_BYTES.len() - 1]);
    let result: String = nstring.into();
    assert_eq!(result, "M".to_string());
}
