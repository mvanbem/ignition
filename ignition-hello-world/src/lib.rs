#![no_std]
#![feature(default_alloc_error_handler)]

extern crate alloc;

use core::panic::PanicInfo;
use core::time::Duration;

use alloc::vec::Vec;
use futures::prelude::stream::StreamExt;
use futures::stream::FuturesUnordered;
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
    log("Allocating timers...");
    let mut timers = Vec::new();
    for _ in 0..100_000 {
        timers.push(sleep(Duration::ZERO));
    }
    executor::spawn(async move {
        let mut f: FuturesUnordered<_> = timers.into_iter().collect();
        while let Some(_) = f.next().await {}
        log("All timers have elapsed");
        shutdown();
    });
}
