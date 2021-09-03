use std::str::from_utf8;

use chrono::{SecondsFormat, Utc};
use wasmtime::{Caller, Extern, Trap};

pub fn abort() -> Result<(), Trap> {
    Err(Trap::new("aborted"))
}

pub fn log(mut caller: Caller<'_, ()>, ptr: u32, len: u32) -> Result<(), Trap> {
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
