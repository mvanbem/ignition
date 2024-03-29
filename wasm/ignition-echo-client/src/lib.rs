use std::str::from_utf8;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use ignition_guest::api::{log, shutdown};
use ignition_guest::rpc_client::RpcClient;
use ignition_guest::runtime::spawn;
use ignition_guest::{emit_wake, Instant};

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
                    (elapsed_seconds * 1e6).ceil(),
                ));

                if shared_state.counter.fetch_sub(1, Ordering::SeqCst) == 1 {
                    shutdown();
                }
            });
        }
    })
}
