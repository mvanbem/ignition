#![no_std]
#![feature(core_intrinsics, default_alloc_error_handler)]

extern crate alloc;
extern crate ignition_guest_panic_abort;

use core::str::from_utf8;
use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::format;
use alloc::sync::Arc;
use ignition_guest::api::{log, shutdown};
use ignition_guest::rpc_client::RpcClient;
use ignition_guest::runtime::spawn;
use ignition_guest::{emit_wake, Instant};
use wee_alloc::WeeAlloc;

#[global_allocator]
static ALLOC: WeeAlloc = WeeAlloc::INIT;

fn ceilf64(x: f64) -> f64 {
    unsafe { core::intrinsics::ceilf64(x) }
}

const MESSAGES: &[&str] = &["abc123", "def456", "ghi789", "hello, world", "asdfjkl;"];

struct SharedState {
    counter: AtomicUsize,
    client: RpcClient,
}

emit_wake!(init);

fn init() {
    spawn(async {
        let shared_state = Arc::new(SharedState {
            counter: AtomicUsize::new(MESSAGES.len()),
            client: RpcClient::new("EchoService"),
        });
        shared_state.client.wait_healthy().await;

        for message in MESSAGES.iter().copied() {
            let shared_state = Arc::clone(&shared_state);
            spawn(async move {
                let start_time = Instant::now();

                let request = shared_state.client.request("echo");
                request.write_all(message.as_bytes()).await;
                let response = request.into_response().read_to_end().await;
                let elapsed_seconds = (Instant::now() - start_time).as_secs_f64();

                assert_eq!(message.as_bytes(), response);
                log(&format!(
                    "Got response {:?} in {} us",
                    from_utf8(&response).unwrap(),
                    ceilf64(elapsed_seconds * 1e6),
                ));

                if shared_state.counter.fetch_sub(1, Ordering::SeqCst) == 1 {
                    shutdown();
                }
            });
        }
    })
}
