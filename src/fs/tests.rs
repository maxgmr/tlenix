#![allow(clippy::unwrap_used)]

use alloc::string::ToString;

use crate::{
    Errno, assert_err, format,
    fs::{FileType, types::DirEntType},
};

use super::*;

const THIS_PATH: &str = "src/fs/file.rs";
const TEST_PATH: &str = "test_files/test.txt";
const SYMLINK_PATH: &str = "test_files/test_symlink";
const TEST_PATH_CONTENTS: &str =
    "Hello! I hope you can read me without any issues! - Max (马克斯)\n";
const TEMP_DIR: &str = "/tmp";
const LARGE_PATH: &str = "test_files/large_file.txt";
const LARGE_CONTENTS_BYTES: [u8; 10_000] = [b'e'; 10_000];

const RENAME_DIR: &str = "/tmp/tlenix_rename_tests";

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
fn append_file() {
    const PATH: &str = "/tmp/append_file";
    const ORIG_CONTENTS: &str = "some random data";
    const OVERWRITE: &str = "XXXX";
    let mut buffer_1 = [0; ORIG_CONTENTS.len()];
    let mut buffer_2 = [0; ORIG_CONTENTS.len() + OVERWRITE.len()];

    let file = OpenOptions::new()
        .read_write()
        .create(true)
        .append(true)
        .open(PATH)
        .unwrap();

    let write_1_result = file.write(ORIG_CONTENTS.as_bytes());
    let seek_1_result = file.set_cursor(0);
    let read_1_result = file.read(&mut buffer_1);
    let seek_2_result = file.set_cursor(0);
    let write_2_result = file.write(OVERWRITE.as_bytes());
    let seek_3_result = file.set_cursor(0);
    let read_2_result = file.read(&mut buffer_2);

    // We need to clean up after ourselves *before* possibly panicking!
    drop(file);
    rm(PATH).unwrap();

    write_1_result.unwrap();
    seek_1_result.unwrap();
    read_1_result.unwrap();
    seek_2_result.unwrap();
    write_2_result.unwrap();
    seek_3_result.unwrap();
    read_2_result.unwrap();

    assert_eq!(buffer_1, ORIG_CONTENTS.as_bytes());
    assert_eq!(
        &buffer_2[..],
        [ORIG_CONTENTS.as_bytes(), OVERWRITE.as_bytes()].concat()
    );
}

#[test_case]
fn o_dir_enotdir() {
    assert_err!(
        OpenOptions::new().directory(true).open(THIS_PATH),
        Errno::Enotdir
    );
}

#[test_case]
fn o_dir() {
    OpenOptions::new().directory(true).open("/").unwrap();
}

#[test_case]
fn o_creat_exist_ok() {
    OpenOptions::new().create(true).open(THIS_PATH).unwrap();
}

#[test_case]
fn o_excl_creat_eexist() {
    assert_err!(
        OpenOptions::new().create_new(true).open(THIS_PATH),
        Errno::Eexist
    );
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
fn other_ops_should_fail_o_path() {
    const PATH: &str = "/tmp/other_ops_should_fail_o_path";
    {
        OpenOptions::new().create(true).open(PATH).unwrap();
    }
    let file = OpenOptions::new().path_only(true).open(PATH).unwrap();
    let mut buffer = [0; 1];
    let read_result = file.read(&mut buffer);
    let write_result = file.write("test".as_bytes());
    drop(file);

    // Clean up after yourself before possibly panicking!
    rm(PATH).unwrap();

    assert_err!(read_result, Errno::Ebadf);
    assert_err!(write_result, Errno::Ebadf);
}

#[test_case]
fn trunc_write() {
    const PATH: &str = "/tmp/trunc_write";
    let file = OpenOptions::new()
        .read_write()
        .create(true)
        .open(PATH)
        .unwrap();

    file.write("test".as_bytes()).unwrap();
    file.set_cursor(0).unwrap();
    let mut buffer = [0; 4];
    file.read(&mut buffer).unwrap();
    assert_eq!("test".as_bytes(), buffer);
    drop(file);

    let file = OpenOptions::new().truncate(true).open(PATH).unwrap();
    buffer = [0xff; 4];
    file.read(&mut buffer).unwrap();
    drop(file);
    rm(PATH).unwrap();
    assert_eq!([0xff; 4], buffer);
}

#[test_case]
fn read_advance_cursor() {
    let mut buffer = [0; 20];
    let file = OpenOptions::new().open(TEST_PATH).unwrap();
    assert_eq!(file.cursor().unwrap().unwrap(), 0);

    let bytes_read = file.read(&mut buffer).unwrap();
    assert_eq!(file.cursor().unwrap().unwrap(), bytes_read);

    let bytes_read = file.read(&mut buffer).unwrap();
    assert_eq!(file.cursor().unwrap().unwrap(), bytes_read * 2);

    let bytes_read = file.read(&mut buffer).unwrap();
    assert_eq!(file.cursor().unwrap().unwrap(), bytes_read * 3);
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
    assert_eq!(file.cursor().unwrap().unwrap(), 0);

    assert_eq!(file.cursor_offset(4).unwrap().unwrap(), 4);
    assert_eq!(file.cursor().unwrap().unwrap(), 4);

    assert_eq!(file.cursor_offset(-2).unwrap().unwrap(), 2);
    assert_eq!(file.cursor().unwrap().unwrap(), 2);

    assert_eq!(file.cursor_offset(10000).unwrap().unwrap(), 10002);
    assert_eq!(file.cursor().unwrap().unwrap(), 10002);
}

#[test_case]
fn file_set_cursor() {
    let file = OpenOptions::new().open(TEST_PATH).unwrap();
    assert_eq!(file.cursor().unwrap().unwrap(), 0);

    assert_eq!(file.set_cursor(200).unwrap().unwrap(), 200);
    assert_eq!(file.cursor().unwrap().unwrap(), 200);

    assert_eq!(file.set_cursor(200).unwrap().unwrap(), 200);
    assert_eq!(file.cursor().unwrap().unwrap(), 200);

    assert_err!(file.set_cursor(-1), Errno::Einval);
}

#[test_case]
fn file_cursor_to_end() {
    let file = OpenOptions::new().open(TEST_PATH).unwrap();
    assert_eq!(file.cursor().unwrap().unwrap(), 0);

    assert_eq!(
        file.cursor_to_end().unwrap().unwrap(),
        TEST_PATH_CONTENTS.len()
    );
    assert_eq!(file.cursor().unwrap().unwrap(), TEST_PATH_CONTENTS.len());
}

#[test_case]
fn file_cursor_end_offset() {
    let file = OpenOptions::new().open(TEST_PATH).unwrap();
    assert_eq!(file.cursor().unwrap().unwrap(), 0);

    assert_eq!(
        file.cursor_to_end_offset(-20).unwrap().unwrap(),
        TEST_PATH_CONTENTS.len() - 20
    );
    assert_eq!(
        file.cursor().unwrap().unwrap(),
        TEST_PATH_CONTENTS.len() - 20
    );

    assert_eq!(
        file.cursor_to_end_offset(50).unwrap().unwrap(),
        TEST_PATH_CONTENTS.len() + 50
    );
    assert_eq!(
        file.cursor().unwrap().unwrap(),
        TEST_PATH_CONTENTS.len() + 50
    );
}

#[test_case]
fn cursor_on_stdin() {
    const STDIN: File = File::define(FileDescriptor::define(0));
    assert!(STDIN.cursor().unwrap().is_none());
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

#[test_case]
fn dir_ents_empty() {
    const DIR: &str = "/tmp/dir_ents_empty";
    const SELF: &str = ".";
    const SUPER: &str = "..";

    mkdir(DIR, FilePermissions::default()).unwrap();

    let dir = OpenOptions::new().directory(true).open(DIR).unwrap();
    let dir_ents_result = dir.dir_ents();
    let is_dir_empty_result = dir.is_dir_empty();

    // Clean up after yourself before testing!
    drop(dir);
    rmdir(DIR).unwrap();

    assert!(is_dir_empty_result.unwrap());

    let dir_ents = dir_ents_result.unwrap();

    // crate::println!("{:#?}", dir_ents);

    assert_eq!(dir_ents.len(), 2);

    let super_dir = dir_ents
        .iter()
        .find(|dir_ent| dir_ent.name == SUPER)
        .unwrap();
    let self_dir = dir_ents
        .iter()
        .find(|dir_ent| dir_ent.name == SELF)
        .unwrap();

    assert_eq!(super_dir.d_type, DirEntType::Dir);
    assert_eq!(self_dir.d_type, DirEntType::Dir);
}

#[test_case]
fn dir_ents_file_and_dir() {
    const DIR: &str = "/tmp/dir_ents_file_and_dir";
    const THIS_DIR: &str = ".";
    const SUPER_DIR: &str = "..";
    const FILE: &str = "my_file";
    const SUBDIR: &str = "my_subdir";

    let mut file_path = DIR.to_string();
    file_path.push('/');
    file_path.push_str(FILE);

    let mut subdir_path = DIR.to_string();
    subdir_path.push('/');
    subdir_path.push_str(SUBDIR);

    mkdir(DIR, FilePermissions::default() | FilePermissions::S_IXUSR).unwrap();

    let dir = OpenOptions::new().directory(true).open(DIR).unwrap();

    // Create file and subdir within dir
    let file = OpenOptions::new()
        .create(true)
        .open(file_path.clone())
        .unwrap();
    mkdir(subdir_path.clone(), FilePermissions::default()).unwrap();

    let dir_ents_result = dir.dir_ents();
    let is_dir_empty_result = dir.is_dir_empty();

    // Clean up after yourself before testing!
    drop(file);
    rm(file_path.clone()).unwrap();
    rmdir(subdir_path.clone()).unwrap();
    rmdir(DIR).unwrap();

    assert!(!is_dir_empty_result.unwrap());

    // Look for the dir, the file, the super dir, and the subdir within the dir ents
    let dir_ents = dir_ents_result.unwrap();

    assert_eq!(dir_ents.len(), 4);

    let this_dir_dent = dir_ents.iter().find(|dent| dent.name == THIS_DIR).unwrap();
    let super_dir_dent = dir_ents.iter().find(|dent| dent.name == SUPER_DIR).unwrap();
    let subdir_dent = dir_ents.iter().find(|dent| dent.name == SUBDIR).unwrap();
    let file_dent = dir_ents.iter().find(|dent| dent.name == FILE).unwrap();

    assert_eq!(this_dir_dent.d_type, DirEntType::Dir);
    assert_eq!(super_dir_dent.d_type, DirEntType::Dir);
    assert_eq!(subdir_dent.d_type, DirEntType::Dir);
    assert_eq!(file_dent.d_type, DirEntType::Reg);
}

#[test_case]
fn is_dir_empty_true() {
    const PATH: &str = "/tmp/is_dir_empty_true";
    mkdir(PATH, FilePermissions::default()).unwrap();
    let is_dir_empty_result = OpenOptions::new()
        .directory(true)
        .open(PATH)
        .unwrap()
        .is_dir_empty();

    // Clean up after yourself before testing!
    rmdir(PATH).unwrap();

    assert!(is_dir_empty_result.unwrap());
}

#[test_case]
fn is_dir_empty_false() {
    assert!(
        !OpenOptions::new()
            .open("/")
            .unwrap()
            .is_dir_empty()
            .unwrap()
    );
}

#[test_case]
fn is_dir_empty_not_dir() {
    assert_err!(
        OpenOptions::new().open(THIS_PATH).unwrap().is_dir_empty(),
        Errno::Enotdir
    );
}

#[test_case]
fn read_to_string() {
    let file_contents = OpenOptions::new()
        .open(TEST_PATH)
        .unwrap()
        .read_to_string()
        .unwrap();
    assert_eq!(file_contents, TEST_PATH_CONTENTS);
}

#[test_case]
fn read_to_string_large() {
    let mut file_contents = OpenOptions::new()
        .open(LARGE_PATH)
        .unwrap()
        .read_to_string()
        .unwrap();
    // Pop off the newline at the end
    assert_eq!(file_contents.pop().unwrap(), '\n');
    assert_eq!(
        file_contents,
        str::from_utf8(&LARGE_CONTENTS_BYTES).unwrap()
    );
}

#[test_case]
fn rename_basic() {
    let path = format!("{RENAME_DIR}/rename_basic_test");
    let expected = format!("{RENAME_DIR}/rename_basic_test_pass");
    // Create dir if it doesn't already exist
    let _ = mkdir(RENAME_DIR, FilePermissions::from(0o777));
    // Make sure file doesn't exist already
    let _ = rm(&path);

    OpenOptions::new().create(true).open(&path).unwrap();
    // Ensure file exists
    OpenOptions::new().open(&path).unwrap();
    rename(&path, &expected).unwrap();
    // Ensure old file name doesn't exist
    assert_err!(OpenOptions::new().open(&path), Errno::Enoent);
    // Ensure new file name does exist
    OpenOptions::new().open(&expected).unwrap();
    // Clean up after yourself
    rm(&expected).unwrap();
}

#[test_case]
fn rename_overwrite() {
    const F1_CONTENTS: &str = "123";
    const F2_CONTENTS: &str = "abc";

    let path = format!("{RENAME_DIR}/rename_overwrite_test");
    let expected = format!("{RENAME_DIR}/rename_overwrite_test_pass");
    // Create dir if it doesn't already exist
    let _ = mkdir(RENAME_DIR, FilePermissions::from(0o777));
    // Make sure file doesn't exist already
    let _ = rm(&path);

    let f1 = OpenOptions::new()
        .read_write()
        .create(true)
        .open(&path)
        .unwrap();
    f1.write(F1_CONTENTS.as_bytes()).unwrap();
    f1.set_cursor(0).unwrap();
    let f2 = OpenOptions::new()
        .read_write()
        .create(true)
        .open(&expected)
        .unwrap();
    f2.write(F2_CONTENTS.as_bytes()).unwrap();
    f2.set_cursor(0).unwrap();
    assert_eq!(&f1.read_to_string().unwrap(), F1_CONTENTS);
    assert_eq!(&f2.read_to_string().unwrap(), F2_CONTENTS);
    drop(f1);
    drop(f2);

    // Overwrite
    rename(&path, &expected).unwrap();

    // Make sure original file name is gone
    assert_err!(OpenOptions::new().open(&path), Errno::Enoent);
    // Make sure overwritten file exists
    let overwritten = OpenOptions::new().open(&expected).unwrap();
    assert_eq!(&overwritten.read_to_string().unwrap(), F1_CONTENTS);

    // Clean up after yourself
    rm(&expected).unwrap();
}

#[test_case]
fn move_files_to_subdir() {
    const F1: &str = "rename_files_to_subdir_file_1";
    const F2: &str = "rename_files_to_subdir_file_2";

    let subdir = format!("{RENAME_DIR}/rename_files_to_subdir_test");
    let f1_orig = format!("{RENAME_DIR}/{F1}");
    let f2_orig = format!("{RENAME_DIR}/{F2}");
    let f1_expected = format!("{subdir}/{F1}");
    let f2_expected = format!("{subdir}/{F2}");

    let _ = mkdir(RENAME_DIR, FilePermissions::from(0o777));

    OpenOptions::new().create(true).open(&f1_orig).unwrap();
    OpenOptions::new().create(true).open(&f2_orig).unwrap();
    let _ = mkdir(&subdir, FilePermissions::from(0o777));

    // Make sure files are there
    OpenOptions::new().open(&f1_orig).unwrap();
    OpenOptions::new().open(&f2_orig).unwrap();

    // Move files
    rename(&f1_orig, format!("{subdir}/{F1}")).unwrap();
    rename(&f2_orig, format!("{subdir}/{F2}")).unwrap();

    // Make sure files are gone from old locations
    assert_err!(OpenOptions::new().open(&f1_orig), Errno::Enoent);
    assert_err!(OpenOptions::new().open(&f2_orig), Errno::Enoent);

    // Make sure files are at new locations
    OpenOptions::new().open(&f1_expected).unwrap();
    OpenOptions::new().open(&f2_expected).unwrap();

    // Clean up after yourself
    rm(&f1_expected).unwrap();
    rm(&f2_expected).unwrap();
    rmdir(&subdir).unwrap();
}

#[test_case]
fn cant_rename_file_to_dir() {
    let f = format!("{RENAME_DIR}/cant_rename_file_to_dir_file");
    let d = format!("{RENAME_DIR}/cant_rename_file_to_dir_dir");

    let _ = mkdir(&d, FilePermissions::from(0o777));
    OpenOptions::new().create(true).open(&f).unwrap();

    assert_err!(rename(&f, &d), Errno::Eisdir);

    // Clean up after yourself!
    rm(f).unwrap();
    rmdir(d).unwrap();
}

#[test_case]
fn cant_rename_dir_to_file() {
    let f = format!("{RENAME_DIR}/cant_rename_file_to_dir_file");
    let d = format!("{RENAME_DIR}/cant_rename_file_to_dir_dir");

    let _ = mkdir(&d, FilePermissions::from(0o777));
    OpenOptions::new().create(true).open(&f).unwrap();

    assert_err!(rename(&d, &f), Errno::Enotdir);

    // Clean up after yourself!
    rm(f).unwrap();
    rmdir(d).unwrap();
}

#[test_case]
fn rename_no_overwrite_full_dir() {
    let d1 = format!("{RENAME_DIR}/rename_no_overwrite_full_dir_d1");
    let d2 = format!("{RENAME_DIR}/rename_no_overwrite_full_dir_d2");
    let f = format!("{d2}/rename_no_overwrite_full_dir_f");

    let _ = mkdir(&d1, FilePermissions::from(0o777));
    let _ = mkdir(&d2, FilePermissions::from(0o777));
    OpenOptions::new().create(true).open(&f).unwrap();

    assert_err!(rename(&d1, &d2), Errno::Enotempty);

    rm(f).unwrap();

    // OK to overwrite now, d2 is empty
    rename(&d1, &d2).unwrap();

    rmdir(d2).unwrap();
}
