use std::sync::Arc;

use chrono::{SecondsFormat, Utc};
use wasmtime::{AsContext, Caller, Trap};

use crate::process::process::Process;
use crate::util::{get_memory, get_str};
use crate::{TaskId, WakeParams};

pub fn shutdown(caller: Caller<'_, Arc<Process>>) {
    caller.data().shutdown();
}

pub fn abort() -> Result<(), Trap> {
    Err(Trap::new("aborted"))
}

pub fn log(mut caller: Caller<'_, Arc<Process>>, ptr: u32, len: u32) -> Result<(), Trap> {
    let memory = get_memory(&mut caller)?;
    let message = get_str(caller.as_context(), memory, ptr, len)?;

    println!(
        "[{}] {} {}",
        Utc::now().to_rfc3339_opts(SecondsFormat::Micros, true),
        caller.data().pid(),
        message,
    );

    Ok(())
}

pub fn impulse(caller: Caller<'_, Arc<Process>>, task_id: u32) {
    let task_id = TaskId(task_id);

    caller
        .data()
        .wake_queue_sender()
        .send(WakeParams { task_id, param: 0 })
        .unwrap();
}
