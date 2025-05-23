#![allow(clippy::unwrap_used)]

use crate::{Errno, assert_err, fs::FileType};

use super::*;

const THIS_PATH: &str = "src/fs/file.rs";
const TEST_PATH: &str = "test_files/test.txt";
const SYMLINK_PATH: &str = "test_files/test_symlink";
const TEST_PATH_CONTENTS: &str =
    "Hello! I hope you can read me without any issues! - Max (马克斯)\n";
const TEMP_DIR: &str = "/tmp";

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

#[test_case]
fn follow_symlink() {
    let mut buffer = [0; TEST_PATH_CONTENTS.len()];
    OpenOptions::new()
        .open(SYMLINK_PATH)
        .unwrap()
        .read(&mut buffer)
        .unwrap();
    assert_eq!(buffer, TEST_PATH_CONTENTS.as_bytes());
}

#[test_case]
fn tempfile() {
    const EXPECTED: [u8; 17] = *b"Howdeedoodeethere";

    let tempfile = OpenOptions::new()
        .read_write()
        .create_temp(true)
        .open(TEMP_DIR)
        .unwrap();

    let bytes_written = tempfile.write(&EXPECTED[..]).unwrap();
    assert_eq!(bytes_written, EXPECTED.len());

    tempfile.set_cursor(0).unwrap();

    let mut buffer = [0; EXPECTED.len() * 2];
    let bytes_read = tempfile.read(&mut buffer).unwrap();
    assert_eq!(bytes_read, EXPECTED.len());
    assert_eq!(&buffer[..EXPECTED.len()], EXPECTED);
}

#[test_case]
fn file_cursor_offset() {
    let file = OpenOptions::new().open(TEST_PATH).unwrap();
    assert_eq!(file.cursor().unwrap(), 0);

    assert_eq!(file.cursor_offset(4).unwrap(), 4);
    assert_eq!(file.cursor().unwrap(), 4);

    assert_eq!(file.cursor_offset(-2).unwrap(), 2);
    assert_eq!(file.cursor().unwrap(), 2);

    assert_eq!(file.cursor_offset(10000).unwrap(), 10002);
    assert_eq!(file.cursor().unwrap(), 10002);
}

#[test_case]
fn file_set_cursor() {
    let file = OpenOptions::new().open(TEST_PATH).unwrap();
    assert_eq!(file.cursor().unwrap(), 0);

    assert_eq!(file.set_cursor(200).unwrap(), 200);
    assert_eq!(file.cursor().unwrap(), 200);

    assert_eq!(file.set_cursor(200).unwrap(), 200);
    assert_eq!(file.cursor().unwrap(), 200);

    assert_err!(file.set_cursor(-1), Errno::Einval);
}

#[test_case]
fn file_cursor_to_end() {
    let file = OpenOptions::new().open(TEST_PATH).unwrap();
    assert_eq!(file.cursor().unwrap(), 0);

    assert_eq!(file.cursor_to_end().unwrap(), TEST_PATH_CONTENTS.len());
    assert_eq!(file.cursor().unwrap(), TEST_PATH_CONTENTS.len());
}

#[test_case]
fn file_cursor_end_offset() {
    let file = OpenOptions::new().open(TEST_PATH).unwrap();
    assert_eq!(file.cursor().unwrap(), 0);

    assert_eq!(
        file.cursor_to_end_offset(-20).unwrap(),
        TEST_PATH_CONTENTS.len() - 20
    );
    assert_eq!(file.cursor().unwrap(), TEST_PATH_CONTENTS.len() - 20);

    assert_eq!(
        file.cursor_to_end_offset(50).unwrap(),
        TEST_PATH_CONTENTS.len() + 50
    );
    assert_eq!(file.cursor().unwrap(), TEST_PATH_CONTENTS.len() + 50);
}

// This test fails if your project directory doesn't end with "tlenix" :/
#[test_case]
fn cwd() {
    const EXPECTED: &str = "tlenix";
    let working_dir = get_cwd().unwrap();
    assert_eq!(&working_dir[working_dir.len() - EXPECTED.len()..], EXPECTED);
}

#[test_case]
fn cd_root() {
    let old_path = get_cwd().unwrap();
    let new_path = "/";

    change_dir(new_path).unwrap();
    let cwd = get_cwd().unwrap();

    // Clean up after yourself!
    change_dir(old_path.as_str()).unwrap();

    assert_eq!(&cwd, new_path);
}

#[test_case]
fn cd_dir_dne() {
    assert_err!(
        change_dir("kefhlskhfsfesg/ezgs/egeg/esgesges/gegesgesg"),
        Errno::Enoent
    );
}

#[test_case]
fn mk_rm_dir() {
    const PATH: &str = "/tmp/mk_rm_dir";
    mkdir(PATH, FilePermissions::default()).unwrap();
    rmdir(PATH).unwrap();
}

#[test_case]
fn mkdir_eexist() {
    assert_err!(mkdir("/", FilePermissions::default()), Errno::Eexist);
}

#[test_case]
fn mkdir_enoent() {
    assert_err!(
        mkdir("gsdjsgehe/fskjnfzljkgnkje", FilePermissions::default()),
        Errno::Enoent
    );
}

#[test_case]
fn rmdir_einval() {
    assert_err!(rmdir("."), Errno::Einval);
}

#[test_case]
fn rmdir_enoent() {
    assert_err!(rmdir("sjgdkjgrelknjr/slghekj"), Errno::Enoent);
}

#[test_case]
fn rmdir_enotdir() {
    assert_err!(rmdir(THIS_PATH), Errno::Enotdir);
}

#[test_case]
fn rmdir_enotempty() {
    assert_err!(rmdir("src"), Errno::Enotempty);
}

#[test_case]
fn mk_rm_file() {
    const PATH: &str = "/tmp/mk_rm_file";
    OpenOptions::new().create(true).open(PATH).unwrap();
    rm(PATH).unwrap();
}

#[test_case]
fn rm_wait_for_last_fd_drop() {
    const PATH: &str = "/tmp/rm_wait_for_last_fd_drop";
    {
        let my_file = OpenOptions::new().create(true).open(PATH).unwrap();
        rm(PATH).unwrap();
        // Ensure able to read from file still
        assert!(my_file.read_byte().unwrap().is_none());
    }
    assert_err!(OpenOptions::new().open(PATH), Errno::Enoent);
}

#[test_case]
fn rm_wait_multiple_fds() {
    const PATH: &str = "/tmp/rm_wait_multiple_fds";
    {
        let fd_1 = OpenOptions::new().create(true).open(PATH).unwrap();
        {
            let fd_2 = OpenOptions::new().open(PATH).unwrap();

            rm(PATH).unwrap();

            assert!(fd_1.read_byte().unwrap().is_none());
            assert!(fd_2.read_byte().unwrap().is_none());
        }
        assert!(fd_1.read_byte().unwrap().is_none());
    }
    assert_err!(OpenOptions::new().open(PATH), Errno::Enoent);
}

#[test_case]
fn rm_eisdir() {
    assert_err!(rm("/"), Errno::Eisdir);
}

#[test_case]
fn rm_enoent_empty() {
    assert_err!(rm(""), Errno::Enoent);
}

#[test_case]
fn rm_enoent_dne() {
    assert_err!(rm("dskjgdskjgnslkjghesg"), Errno::Enoent);
}
