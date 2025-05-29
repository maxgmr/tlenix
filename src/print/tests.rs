use alloc::string::{String, ToString};
use core::fmt::Display;

// Used for debug print testing.
#[derive(Debug)]
#[allow(clippy::struct_field_names)]
struct MyTestStruct {
    my_number: i32,
    my_string: String,
    my_str: &'static str,
    my_tuple: (char, f32),
}
impl MyTestStruct {
    /// Creates a basic [`MyTestStruct`] to test printing.
    fn example() -> Self {
        Self {
            my_number: -42,
            my_string: "hello there!".to_string(),
            my_str: "awhaahwahwhah",
            my_tuple: ('福', 79.2335),
        }
    }
}
impl Display for MyTestStruct {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "my_number is {}, my string is {}, my str is {}, and my tuple is {:?}.",
            self.my_number, self.my_string, self.my_str, self.my_tuple
        )
    }
}

#[test_case]
fn print_str() {
    print!("helloe :D");
    println!("您们好, 我是马克斯 :)");
    eprint!("all good");
    eprintln!("hooray!");
}

#[test_case]
fn print_string() {
    let my_string: String = "this is a test string.".to_string();
    print!("{my_string}");
}

#[test_case]
fn fmt_str() {
    print!("My test works {}.", "well");
}

#[test_case]
fn fmt_int() {
    print!("{} + {} = {}", 1, 1, 2);
}

#[test_case]
fn fmt_int_math() {
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
fn debug_fmt() {
    println!("{:?}", MyTestStruct::example());
}

#[test_case]
fn pretty_debug() {
    println!("{:#?}", MyTestStruct::example());
}

#[test_case]
fn display_impl() {
    print!("{}", MyTestStruct::example());
}

#[test_case]
fn format_empty() {
    assert_eq!(format!(""), "");
}

#[test_case]
fn format_literal() {
    assert_eq!(format!("abc123"), "abc123");
}

#[test_case]
fn format_subst() {
    assert_eq!(format!("{}+{}={}", 1, 1, 1 + 1), "1+1=2");
}

#[test_case]
fn format_width() {
    const EXPECTED: &str = "Hello x    !";
    assert_eq!(format!("Hello {:5}!", "x"), EXPECTED);
    assert_eq!(format!("Hello {:1$}!", "x", 5), EXPECTED);
    assert_eq!(format!("Hello {1:0$}!", 5, "x"), EXPECTED);
}
