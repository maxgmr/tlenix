//! Custom test framework for `tlenix_core` tests.

#[cfg(test)]
use core::panic::PanicInfo;

#[cfg(test)]
use crate::eprintln;
use crate::{print, println};

/// [`Testable`] types can be run as tests and should panic if their test fails.
pub trait Testable {
    /// Run the test and panic on failure.
    fn run(&self);
}
impl<T: Fn()> Testable for T {
    fn run(&self) {
        print!("{}...\t", core::any::type_name::<T>());
        self();
        println!("[ok]");
    }
}

/// The custom test framework's test runner.
pub fn custom_test_runner(tests: &[&dyn Testable]) {
    println!("Running {} test(s)...", tests.len());
    println!("=======");
    for test in tests {
        test.run();
    }
    println!("\n=======");
    println!(
        "[ALL_PASS] All {} test(s) passed successfully! :D",
        tests.len()
    );
}

/// Display failure and panic message.
#[cfg(test)]
pub fn test_panic_handler(info: &PanicInfo<'_>) -> ! {
    eprintln!("[FAIL!]");
    eprintln!("Error:\n{}", info);

    // TODO use a better loop
    #[allow(clippy::empty_loop)]
    loop {}
}
