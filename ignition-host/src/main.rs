use wasmtime::{Engine, Linker, Module, Store};

mod api;

fn main() -> anyhow::Result<()> {
    let engine = Engine::default();
    let module = Module::from_file(&engine, "target/wasm32-unknown-unknown/release/opt.wasm")?;

    let mut linker = Linker::new(&engine);
    linker.func_wrap("ignition", "abort", api::abort)?;
    linker.func_wrap("ignition", "log", api::log)?;

    let mut store = Store::new(&engine, ());
    let instance = linker.instantiate(&mut store, &module)?;
    let main = instance.get_typed_func::<(), (), _>(&mut store, "main")?;

    main.call(&mut store, ())?;

    println!("main() returned");

    Ok(())
}
