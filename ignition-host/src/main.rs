use wasmtime::{Engine, Linker, Module, Store};

use crate::state::State;

mod api;
mod state;

#[derive(Debug)]
pub struct TaskId(pub u32);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let engine = Engine::default();
    let module = Module::from_file(&engine, "target/wasm32-unknown-unknown/release/opt.wasm")?;

    let mut linker = Linker::new(&engine);
    linker.func_wrap("ignition", "shutdown", api::shutdown)?;
    linker.func_wrap("ignition", "abort", api::abort)?;
    linker.func_wrap("ignition", "log", api::log)?;
    linker.func_wrap("ignition", "impulse", api::impulse)?;
    linker.func_wrap("ignition", "sleep", api::sleep)?;
    linker.func_wrap("ignition", "monotonic_time", api::monotonic_time)?;

    let (state, mut wake_queue_receiver) = State::new();
    let mut store = Store::new(&engine, state);

    let instance = linker.instantiate(&mut store, &module)?;
    let wake = instance.get_typed_func::<u32, (), _>(&mut store, "wake")?;

    // Make the initial call.
    wake.call(&mut store, !0)?;

    // Dispatch wake events.
    while !store.data().is_shutdown() {
        while let Some(task_id) = store.data_mut().impulse_queue_mut().pop_front() {
            wake.call(&mut store, task_id.0)?;
        }

        if store.data().is_shutdown() {
            break;
        }

        let task_id = wake_queue_receiver.recv().await.unwrap();
        wake.call(&mut store, task_id.0)?;
    }

    Ok(())
}
