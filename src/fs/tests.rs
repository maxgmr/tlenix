#![allow(clippy::unwrap_used)]

use crate::{Errno, fs::FileType};

use super::*;

const THIS_PATH: &str = "src/fs/file.rs";
const TEST_PATH: &str = "test_files/test.txt";
const TEST_PATH_CONTENTS: &str =
    "Hello! I hope you can read me without any issues! - Max (马克斯)\n";

#[test_case]
fn read_bytes() {
    const EXPECTED_STR: &str = "//! This module is";
    let expected = EXPECTED_STR.as_bytes();

    let mut buffer = [0; EXPECTED_STR.len()];
    let bytes_read = OpenOptions::new()
        .open(THIS_PATH)
        .unwrap()
        .read(&mut buffer)
        .unwrap();

    assert_eq!(bytes_read, EXPECTED_STR.len());
    assert_eq!(expected, buffer);
}

#[test_case]
fn read_utf8() {
    let mut buffer = [0; TEST_PATH_CONTENTS.len()];
    let bytes_read = OpenOptions::new()
        .read_write()
        .open(TEST_PATH)
        .unwrap()
        .read(&mut buffer)
        .unwrap();

    assert_eq!(bytes_read, TEST_PATH_CONTENTS.len());
    assert_eq!(TEST_PATH_CONTENTS, str::from_utf8(&buffer).unwrap());
}

#[test_case]
fn read_past_end() {
    let mut buffer = [0; TEST_PATH_CONTENTS.len() - 1];
    let file = OpenOptions::new().open(TEST_PATH).unwrap();
    let bytes_read = file.read(&mut buffer).unwrap();
    let expected = &TEST_PATH_CONTENTS.as_bytes()[..TEST_PATH_CONTENTS.len() - 1];
    assert_eq!(bytes_read, buffer.len());
    assert_eq!(buffer, expected);

    // Attempt to read past the end
    let bytes_read = file.read(&mut buffer).unwrap();
    let mut expected_2 = [0; TEST_PATH_CONTENTS.len() - 1];
    expected_2.copy_from_slice(expected);
    expected_2[0] = b'\n';
    assert_eq!(bytes_read, 1);
    assert_eq!(buffer, expected_2);

    let bytes_read = file.read(&mut buffer).unwrap();
    assert_eq!(bytes_read, 0);
    assert_eq!(buffer, expected_2);
}

#[test_case]
fn read_wo() {
    let mut buffer = [0; 1];
    let file = OpenOptions::new().write_only().open(TEST_PATH).unwrap();

    match file.read(&mut buffer) {
        Err(Errno::Ebadf) => {} // OK!
        val => panic!("expected Err(Errno::Ebadf), got {val:?}"),
    }

    match file.read_byte() {
        Err(Errno::Ebadf) => {} // OK!
        val => panic!("expected Err(Errno::Ebadf), got {val:?}"),
    }
}

#[test_case]
fn read_dir() {
    let mut buffer = [0; 1];
    match OpenOptions::new().open("/").unwrap().read(&mut buffer) {
        Err(Errno::Eisdir) => {} // OK!
        val => panic!("expected Err(Errno::Eisdir), got {val:?}"),
    }
}

#[test_case]
fn write_ro() {
    let buffer = *b"irrelevant";
    let byte = b'e';

    let file = OpenOptions::new().open(TEST_PATH).unwrap();

    match file.write(&buffer) {
        Err(Errno::Ebadf) => {} // OK!
        val => panic!("expected Err(Errno::Ebadf), got {val:?}"),
    }

    match file.write_byte(byte) {
        Err(Errno::Ebadf) => {} // OK!
        val => panic!("expected Err(Errno::Ebadf), got {val:?}"),
    }
}

#[test_case]
fn stats() {
    let stats = OpenOptions::new().open(TEST_PATH).unwrap().stat().unwrap();
    // crate::println!("{:#?}", stats);
    assert_eq!(stats.file_type, FileType::RegularFile);
    assert_eq!(stats.file_stat_raw.st_size, 68);
}

#[test_case]
fn read_advance_cursor() {
    let mut buffer = [0; 20];
    let file = OpenOptions::new().open(TEST_PATH).unwrap();
    assert_eq!(file.cursor().unwrap(), 0);

    let bytes_read = file.read(&mut buffer).unwrap();
    assert_eq!(file.cursor().unwrap(), bytes_read);

    let bytes_read = file.read(&mut buffer).unwrap();
    assert_eq!(file.cursor().unwrap(), bytes_read * 2);

    let bytes_read = file.read(&mut buffer).unwrap();
    assert_eq!(file.cursor().unwrap(), bytes_read * 3);
}

#[test_case]
fn read_byte() {
    const EXP_LEN: usize = TEST_PATH_CONTENTS.len();

    let file = OpenOptions::new().open(TEST_PATH).unwrap();

    // Read the file's bytes one at a time
    for i in 0..EXP_LEN {
        let byte = file.read_byte().unwrap().unwrap();
        assert_eq!(byte, TEST_PATH_CONTENTS.as_bytes()[i]);
    }

    // Make sure that we get `None` after reading to the end
    assert!(file.read_byte().unwrap().is_none());
}
