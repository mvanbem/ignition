#![no_std]
#![feature(core_intrinsics, default_alloc_error_handler)]

extern crate alloc;

use core::panic::PanicInfo;

use alloc::format;
use alloc::vec::Vec;
use futures::prelude::stream::StreamExt;
use futures::stream::FuturesUnordered;
use ignition_guest::api::{abort, impulse, log, shutdown};
use ignition_guest::runtime::spawn;
use ignition_guest::{emit_wake, Instant};
use wee_alloc::WeeAlloc;

#[global_allocator]
static ALLOC: WeeAlloc = WeeAlloc::INIT;

#[panic_handler]
fn handle_panic(_: &PanicInfo) -> ! {
    abort()
}

fn ceilf64(x: f64) -> f64 {
    unsafe { core::intrinsics::ceilf64(x) }
}

emit_wake!(init);

fn init() {
    const COUNT: usize = 1_000_000;

    let start_time = Instant::now();
    let mut impulses = Vec::new();
    for _ in 0..COUNT {
        impulses.push(impulse());
    }
    spawn(async move {
        let mut f: FuturesUnordered<_> = impulses.into_iter().collect();
        while let Some(_) = f.next().await {}
        let elapsed_seconds = (Instant::now() - start_time).as_secs_f64();
        log(&format!(
            "Elapsed: {} s, {} ns per impulse",
            elapsed_seconds,
            ceilf64(elapsed_seconds * (1e9 / COUNT as f64)),
        ));
        shutdown();
    });
}
