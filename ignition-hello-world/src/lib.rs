#![no_std]
#![feature(default_alloc_error_handler)]

extern crate alloc;

use core::panic::PanicInfo;
use core::time::Duration;

use ignition_api::{abort, emit_wake, executor, log, shutdown, sleep};
use wee_alloc::WeeAlloc;

#[global_allocator]
static ALLOC: WeeAlloc = WeeAlloc::INIT;

#[panic_handler]
fn handle_panic(_: &PanicInfo) -> ! {
    abort()
}

emit_wake!(init);

fn init() {
    log("Waiting one second");
    executor::spawn(async {
        sleep(Duration::from_secs(1)).await;
        log("Woke up");
        shutdown();
        log("Requested shutdown");
    });
}
