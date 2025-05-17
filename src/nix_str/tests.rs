use super::*;

const TEST_STR: &str = "Hello, world!";
const TEST_ALREADY_NULL_TERM: &str = "Hello, world!\0";
const TEST_EMPTY: &str = "";
const TEST_BYTES: [u8; 13] = *b"Hello, world!";

#[test_case]
fn nstring_from_string() {
    let string = TEST_STR.to_string();
    let expected_bytes = TEST_ALREADY_NULL_TERM.as_bytes();
    let my_nstring: NixString = string.into();
    assert_eq!(my_nstring.bytes(), expected_bytes);
}

#[test_case]
fn nstring_from_byte_vec() {
    let bytes = Vec::from(TEST_STR.as_bytes());
    let expected_bytes = TEST_ALREADY_NULL_TERM.as_bytes();
    let my_nstring: NixString = bytes.into();
    assert_eq!(my_nstring.bytes(), expected_bytes);
}

#[test_case]
fn null_nstring() {
    let expected_bytes = *b"\0";
    let my_nstring = NixString::null();
    assert_eq!(my_nstring.bytes(), expected_bytes);
}
