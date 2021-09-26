use std::env::args;
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Result};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use tokio::spawn;
use wasmtime::{Engine, Linker, Module, Store};

use crate::process::state::ProcessState;

mod api;
mod interop;
mod process;
mod util;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskId(pub u32);

#[derive(Clone, Copy, Debug)]
pub struct WakeParams {
    pub task_id: TaskId,
    pub param: u32,
}

impl WakeParams {
    const INIT: WakeParams = WakeParams {
        task_id: TaskId(!0),
        param: 0,
    };
}

impl From<WakeParams> for (u32, u32) {
    fn from(params: WakeParams) -> Self {
        (params.task_id.0, params.param)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let engine = Engine::default();
    let args = args().skip(1);
    if args.len() == 0 {
        return Err(anyhow!("expected one or more paths to Wasm modules"));
    }

    let mut modules: FuturesUnordered<_> = args
        .enumerate()
        .map(|(pid, path)| {
            let engine = engine.clone();
            spawn(async move { start_module(&engine, &path, pid).await })
        })
        .collect();
    while let Some(result) = modules.next().await {
        result??;
    }

    Ok(())
}

async fn start_module(engine: &Engine, path: &str, pid: usize) -> Result<()> {
    println!("pid {}: Loading {}", pid, path);

    let module = Module::from_file(engine, path)?;

    let mut linker = Linker::new(engine);
    linker.func_wrap("ignition", "shutdown", api::core::shutdown)?;
    linker.func_wrap("ignition", "abort", api::core::abort)?;
    linker.func_wrap("ignition", "log", api::core::log)?;
    linker.func_wrap("ignition", "impulse", api::core::impulse)?;
    linker.func_wrap("ignition", "sleep", api::time::sleep)?;
    linker.func_wrap("ignition", "monotonic_time", api::time::monotonic_time)?;
    linker.func_wrap("ignition", "io_read", api::io::io_read)?;
    linker.func_wrap("ignition", "io_write", api::io::io_write)?;
    linker.func_wrap("ignition", "io_close", api::io::io_close)?;
    linker.func_wrap(
        "ignition",
        "rpc_client_create",
        api::rpc_client::rpc_client_create,
    )?;
    linker.func_wrap(
        "ignition",
        "rpc_client_wait_healthy",
        api::rpc_client::rpc_client_wait_healthy,
    )?;
    linker.func_wrap(
        "ignition",
        "rpc_client_request",
        api::rpc_client::rpc_client_request,
    )?;
    linker.func_wrap(
        "ignition",
        "rpc_server_create",
        api::rpc_server::rpc_server_create,
    )?;
    linker.func_wrap(
        "ignition",
        "rpc_server_get_request",
        api::rpc_server::rpc_server_get_request,
    )?;

    let (state, mut wake_queue_receiver) = ProcessState::new(pid);
    state.wake_queue_sender().send(WakeParams::INIT).unwrap();
    let mut store = Store::new(engine, Arc::new(Mutex::new(state)));

    let instance = linker.instantiate(&mut store, &module)?;
    let wake = instance.get_typed_func::<(u32, u32), (), _>(&mut store, "wake")?;

    // Dispatch wake events.
    while !store.data().lock().unwrap().is_shutdown() {
        let params = wake_queue_receiver.recv().await.unwrap();
        wake.call(&mut store, params.into())?;
    }

    println!("pid {}: Quit", pid);

    Ok(())
}
