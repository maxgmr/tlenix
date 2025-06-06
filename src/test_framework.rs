//! Custom test framework for `tlenix_core` tests.

use crate::{print, println};

/// [`Testable`] types can be run as tests and should panic if their test fails.
pub trait Testable {
    /// Runs the test, panicking on failure.
    fn run(&self);
}
impl<T: Fn()> Testable for T {
    fn run(&self) {
        print!("{}...\t", core::any::type_name::<T>());
        self();
        println!("[\u{001b}[32mok\u{001b}[0m]");
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
        "[\u{001b}[32mALL_PASS\u{001b}[0m] All {} test(s) passed successfully! :D",
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

    eprintln!("[\u{001b}[31mFAIL!\u{001b}[0m]");
    eprintln!("Error:\n{}", info);

    exit(ExitFailure(1));
}
