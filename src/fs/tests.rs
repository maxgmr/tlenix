#![allow(clippy::unwrap_used)]

use crate::{Errno, assert_err, fs::FileType};

use super::*;

const THIS_PATH: &str = "src/fs/file.rs";
const TEST_PATH: &str = "test_files/test.txt";
const SYMLINK_PATH: &str = "test_files/test_symlink";
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

    assert_err!(file.read(&mut buffer), Errno::Ebadf);
    assert_err!(file.read(&mut buffer), Errno::Ebadf);
}

#[test_case]
fn read_dir() {
    let mut buffer = [0; 1];
    assert_err!(
        OpenOptions::new().open("/").unwrap().read(&mut buffer),
        Errno::Eisdir
    );
}

#[test_case]
fn write_ro() {
    let buffer = *b"irrelevant";
    let byte = b'e';

    let file = OpenOptions::new().open(TEST_PATH).unwrap();

    assert_err!(file.write(&buffer), Errno::Ebadf);
    assert_err!(file.write_byte(byte), Errno::Ebadf);
}

#[test_case]
fn stats() {
    let stats = OpenOptions::new().open(TEST_PATH).unwrap().stat().unwrap();
    // crate::println!("{:#?}", stats);
    assert_eq!(stats.file_type, FileType::RegularFile);
    assert_eq!(
        stats.file_stat_raw.st_size,
        TEST_PATH_CONTENTS.len().try_into().unwrap()
    );
}

#[test_case]
fn dir_stats() {
    let stats = OpenOptions::new()
        .path_only(true)
        .open("/")
        .unwrap()
        .stat()
        .unwrap();
    assert_eq!(stats.file_type, FileType::Directory);
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
    let file = OpenOptions::new().open(TEST_PATH).unwrap();

    // Read the file's bytes one at a time
    for i in 0..TEST_PATH_CONTENTS.len() {
        let byte = file.read_byte().unwrap().unwrap();
        assert_eq!(byte, TEST_PATH_CONTENTS.as_bytes()[i]);
    }

    // Make sure that we get `None` after reading to the end
    assert!(file.read_byte().unwrap().is_none());
}
