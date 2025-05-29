//! Custom test framework for `tlenix_core` tests.

use crate::{print, println};

/// Ideal width of a test message.
const SCREEN_COLS: usize = 80;
/// Bit to print after the test starts.
const ELLIPSIS: &str = "...";

/// String to print after a successful test.
const OK_TEXT: &str = "[\u{001b}[32mok\u{001b}[0m]";

/// [`Testable`] types can be run as tests and should panic if their test fails.
pub trait Testable {
    /// Runs the test, panicking on failure.
    fn run(&self);
}
impl<T: Fn()> Testable for T {
    fn run(&self) {
        let initial_text = core::any::type_name::<T>();
        let total_length = initial_text.len() + ELLIPSIS.len() + OK_TEXT.len();
        let padding = if total_length < SCREEN_COLS {
            SCREEN_COLS - total_length
        } else {
            1
        };
        print!("{initial_text}{ELLIPSIS}{: <padding$}", padding);
        self();
        println!("{OK_TEXT}");
    }
}

/// The custom test framework's test runner.
pub fn custom_test_runner(tests: &[&dyn Testable]) {
    println!("Running {} tests...", tests.len());
    println!("=======");
    for test in tests {
        test.run();
    }
    println!("\n=======");
    println!(
        "[\u{001b}[32mSUCCESS\u{001b}[0m] All {} test(s) passed successfully! :D",
        tests.len()
    );
}

/// Display failure and panic message.
#[cfg(test)]
pub fn test_panic_handler(info: &core::panic::PanicInfo<'_>) -> ! {
    use crate::{
        eprintln,
        process::{ExitStatus::ExitFailure, exit},
    };

    eprintln!("[\u{001b}[31mFAIL\u{001b}[0m]");
    eprintln!("Error:\n{}", info);

    exit(ExitFailure(1));
}
