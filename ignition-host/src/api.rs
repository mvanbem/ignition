use std::str::from_utf8;
use std::time::Duration;

use chrono::{SecondsFormat, Utc};
use wasmtime::{Caller, Extern, Trap};

use crate::{State, TaskId};

pub fn shutdown(mut caller: Caller<'_, State>) {
    caller.data_mut().shutdown();
}

pub fn abort() -> Result<(), Trap> {
    Err(Trap::new("aborted"))
}

pub fn log(mut caller: Caller<'_, State>, ptr: u32, len: u32) -> Result<(), Trap> {
    let memory = match caller.get_export("memory") {
        Some(Extern::Memory(memory)) => memory,
        _ => return Err(Trap::new("failed to find memory")),
    };

    let data = memory
        .data(&caller)
        .get(ptr as usize..)
        .and_then(|arr| arr.get(..len as usize))
        .ok_or_else(|| Trap::new("data out of bounds"))?;
    let message = from_utf8(data).map_err(|_| Trap::new("invalid utf-8"))?;

    println!(
        "[{}] {}",
        Utc::now().to_rfc3339_opts(SecondsFormat::Micros, true),
        message,
    );

    Ok(())
}

pub fn sleep(mut caller: Caller<'_, State>, task_id: u32, usec: u32) {
    let task_id = TaskId(task_id);
    let duration = Duration::from_micros(usec.into());

    let wake_queue_sender = caller.data_mut().wake_queue_sender().clone();
    tokio::spawn(async move {
        tokio::time::sleep(duration).await;
        wake_queue_sender.send(task_id).await.unwrap();
    });
}