#![allow(clippy::unwrap_used)]

use super::*;

const SELF_PATH: &str = "src/fs/tests.rs";

#[test_case]
fn open_file() {
    let _ = open(SELF_PATH, &OpenFlags::O_RDONLY).unwrap();
}

fn expect_eperm_helper(open_flags: &OpenFlags) {
    match open("/", open_flags) {
        Err(Errno::Eperm) => (), // OK!
        _ => panic!("expected Err(Errno::Eperm)"),
    }
}

#[test_case]
fn open_no_creat() {
    expect_eperm_helper(&OpenFlags::O_CREAT);
}

#[test_case]
fn open_no_rdonly_trunc() {
    expect_eperm_helper(&(OpenFlags::O_RDONLY | OpenFlags::O_TRUNC));
}
