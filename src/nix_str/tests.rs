#![allow(clippy::unwrap_used)]

use super::*;

const TEST_STR: &str = "Hello, world!";
const TEST_NULL_TERM: &str = "Hello, world!\0";
const TEST_EMPTY: &str = "";
const TEST_BYTES: [u8; 13] = *b"Hello, world!";

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
    let my_nstring: NixString = bytes.into();
    assert_eq!(my_nstring.bytes(), expected_bytes);
}

#[test_case]
fn nstring_from_byte_slice() {
    let my_nstring = NixString::from(&TEST_BYTES[..]);
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
