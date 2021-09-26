use std::convert::TryInto;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use wasmtime::Caller;

use crate::{ProcessState, TaskId, WakeParams};

pub fn sleep(caller: Caller<'_, Arc<Mutex<ProcessState>>>, task_id: u32, usec: u32) {
    let task_id = TaskId(task_id);
    let duration = Duration::from_micros(usec.into());

    let wake_queue_sender = caller.data().lock().unwrap().wake_queue_sender().clone();
    tokio::spawn(async move {
        tokio::time::sleep(duration).await;
        wake_queue_sender
            .send(WakeParams { task_id, param: 0 })
            .unwrap();
    });
}

pub fn monotonic_time(caller: Caller<'_, Arc<Mutex<ProcessState>>>) -> u64 {
    (Instant::now() - caller.data().lock().unwrap().start_time())
        .as_micros()
        .try_into()
        .unwrap()
}
