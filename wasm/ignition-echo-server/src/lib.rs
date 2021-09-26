#![no_std]
#![feature(core_intrinsics, default_alloc_error_handler)]

extern crate alloc;
extern crate ignition_guest_panic_abort;

use core::sync::atomic::{AtomicUsize, Ordering};
use core::time::Duration;

use alloc::boxed::Box;
use alloc::sync::Arc;
use ignition_guest::api::{shutdown, sleep};
use ignition_guest::emit_wake;
use ignition_guest::rpc_server::RpcServerBuilder;
use ignition_guest::runtime::spawn;
use wee_alloc::WeeAlloc;

#[global_allocator]
static ALLOC: WeeAlloc = WeeAlloc::INIT;

emit_wake!(init);

fn init() {
    spawn(async {
        sleep(Duration::from_secs(1)).await;

        let counter = Arc::new(AtomicUsize::new(5));
        RpcServerBuilder::new("EchoService")
            .add_handler(
                "echo",
                Box::new(move |request, response| {
                    let counter = Arc::clone(&counter);
                    Box::pin(async move {
                        response.write_all(&request.read_to_end().await).await;
                        if counter.fetch_sub(1, Ordering::SeqCst) == 1 {
                            shutdown();
                        }
                    })
                }),
            )
            .build();
    });
}
