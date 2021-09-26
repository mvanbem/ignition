#![no_std]

use core::panic::PanicInfo;

use ignition_guest::api::abort;

#[panic_handler]
fn handle_panic(_: &PanicInfo) -> ! {
    abort()
}
