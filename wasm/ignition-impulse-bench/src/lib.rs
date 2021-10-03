use futures::prelude::stream::StreamExt;
use futures::stream::FuturesUnordered;
use ignition_guest::api::{impulse, log, shutdown};
use ignition_guest::runtime::spawn;
use ignition_guest::{emit_wake, Instant};

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
        while f.next().await.is_some() {}
        let elapsed_seconds = (Instant::now() - start_time).as_secs_f64();
        log(&format!(
            "Elapsed: {} s, {} ns per impulse",
            elapsed_seconds,
            (elapsed_seconds * (1e9 / COUNT as f64)).ceil(),
        ));
        shutdown();
    });
}
