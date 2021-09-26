use std::sync::{Arc, Mutex};

use chrono::{SecondsFormat, Utc};
use wasmtime::{AsContext, Caller, Trap};

use crate::process::state::ProcessState;
use crate::util::{get_memory, get_str};
use crate::{TaskId, WakeParams};

pub fn shutdown(caller: Caller<'_, Arc<Mutex<ProcessState>>>) {
    caller.data().lock().unwrap().shutdown();
}

pub fn abort() -> Result<(), Trap> {
    Err(Trap::new("aborted"))
}

pub fn log(
    mut caller: Caller<'_, Arc<Mutex<ProcessState>>>,
    ptr: u32,
    len: u32,
) -> Result<(), Trap> {
    let memory = get_memory(&mut caller)?;
    let message = get_str(caller.as_context(), memory, ptr, len)?;

    println!(
        "[{}] {} {}",
        Utc::now().to_rfc3339_opts(SecondsFormat::Micros, true),
        // TODO: Move the Arc inside so immutable getters like pid() don't take the lock.
        caller.data().lock().unwrap().pid(),
        message,
    );

    Ok(())
}

pub fn impulse(caller: Caller<'_, Arc<Mutex<ProcessState>>>, task_id: u32) {
    let task_id = TaskId(task_id);

    caller
        .data()
        .lock()
        .unwrap()
        .wake_queue_sender()
        .send(WakeParams { task_id, param: 0 })
        .unwrap();
}
