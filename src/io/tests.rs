use crate::{eprint, eprintln, print, println};

// Used for debug print testing.
#[allow(dead_code)]
#[derive(Debug)]
struct MyTestStruct {
    number: i32,
    string: &'static str,
    tuple: (char, f32),
}

const TEST_STRUCT: MyTestStruct = MyTestStruct {
    number: -42,
    string: "hello!",
    tuple: ('M', -0.49),
};

#[test_case]
fn print_lit() {
    print!("helloe :D");
}

#[test_case]
fn str_fmt() {
    print!("My test is {}.", "good");
}

#[test_case]
fn int_fmt() {
    print!("{} + {} = {}", 1, 1, 2);
}

#[test_case]
fn int_math_fmt() {
    print!("{} - {} = {}", 3, 5, 3 - 5);
}

#[test_case]
fn f32_fmt() {
    print!("pi ~= {}", core::f32::consts::PI);
}

#[test_case]
fn f64_fmt() {
    print!("e ~= {}", core::f64::consts::E);
}

#[test_case]
fn eprint() {
    eprint!("e != {}", 4.0);
}

#[test_case]
fn println() {
    println!("hullo!");
}

#[test_case]
fn eprintln() {
    eprintln!("bye!");
}

#[test_case]
fn debug_fmt() {
    println!("{:?}", TEST_STRUCT);
}

#[test_case]
fn pretty_debug() {
    println!("{:#?}", TEST_STRUCT);
}

#[test_case]
fn ident_fmt() {
    let val = 4;
    print!("val={val}, ");
    print!("valB={val2}", val2 = 4);
}

#[test_case]
fn leading_zero() {
    print!("{:04}", 42);
}

#[test_case]
fn radix_fmt() {
    let val = 42;
    print!("{:#x} = {:#b} = {:#o}", val, val, val);
}

#[test_case]
fn uni_fmt() {
    let inf: char = '∞';
    print!("{inf}");
}

#[test_case]
fn cn_fmt() {
    print!("您们好, 我是马克斯 :)");
}

#[test_case]
fn temp_fail() {
    panic!();
}
