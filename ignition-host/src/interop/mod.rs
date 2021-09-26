use wasmtime::{Memory, StoreContext, Trap};

pub mod rpc;

pub trait Wasm {
    const SIZE: u32;
}

pub trait FromWasm: Wasm {
    fn from_wasm<T>(
        context: StoreContext<T>,
        memory: Memory,
        data: &mut &[u8],
    ) -> Result<Self, Trap>
    where
        Self: Sized;
}

pub trait ToWasm: Wasm {
    fn to_wasm(&self, data: &mut &mut [u8]) -> Result<(), Trap>;
}
