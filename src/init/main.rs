#![no_std]
#![no_main]

use core::panic::PanicInfo;

use tlenix_core::{eprint, eprintln, print, println};

const WELCOME_MSG: &str = concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION"));
const TLENIX_PANIC_TITLE: &str = "tlenix";

/// Entry point.
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    welcome_msg();

    panic!("I'm panicking because of how awesome things are!");
    loop {}
}

fn welcome_msg() {
    println!("{}", WELCOME_MSG);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    eprintln!("{} {}", TLENIX_PANIC_TITLE, info);

    // Halt system
    loop {}
}
