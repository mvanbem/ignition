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
    linker.func_wrap("ignition", "sleep", api::sleep)?;

    let (state, mut wake_queue_receiver) = State::new();
    let mut store = Store::new(&engine, state);

    let instance = linker.instantiate(&mut store, &module)?;
    let wake = instance.get_typed_func::<u32, (), _>(&mut store, "wake")?;

    // Make the initial call.
    println!("initial call to wake()");
    wake.call(&mut store, !0)?;
    println!("initial wake() returned");

    // Dispatch wake events.
    while !store.data().is_shutdown() {
        let task_id = wake_queue_receiver.recv().await.unwrap();
        println!("calling wake() for task_id {}", task_id.0);
        wake.call(&mut store, task_id.0)?;
    }

    Ok(())
}
