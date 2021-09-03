#![no_std]

use core::panic::PanicInfo;

use crate::ignition::{abort, log};

mod ignition;

#[panic_handler]
fn handle_panic(_: &PanicInfo) -> ! {
    abort()
}

#[no_mangle]
pub extern "C" fn main() {
    log("main says hello");
}
