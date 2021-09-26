use std::str::from_utf8;

use wasmtime::{AsContext, Caller, Extern, Memory, StoreContext, StoreContextMut, Trap};

pub mod pointer_identity_arc;

pub fn get_memory<T>(caller: &mut Caller<T>) -> Result<Memory, Trap> {
    match caller.get_export("memory") {
        Some(Extern::Memory(memory)) => Ok(memory),
        _ => Err(Trap::new("failed to find memory")),
    }
}

pub fn get_slice<T>(
    context: StoreContext<T>,
    memory: Memory,
    ptr: u32,
    len: u32,
) -> Result<&[u8], Trap> {
    memory
        .data(context)
        .get(ptr as usize..)
        .and_then(|arr| arr.get(..len as usize))
        .ok_or_else(|| Trap::new("data out of bounds"))
}

pub fn get_slice_mut<T>(
    context: StoreContextMut<T>,
    memory: Memory,
    ptr: u32,
    len: u32,
) -> Result<&mut [u8], Trap> {
    memory
        .data_mut(context)
        .get_mut(ptr as usize..)
        .and_then(|arr| arr.get_mut(..len as usize))
        .ok_or_else(|| Trap::new("data out of bounds"))
}

pub fn get_str<T>(
    context: StoreContext<T>,
    memory: Memory,
    ptr: u32,
    len: u32,
) -> Result<&str, Trap> {
    from_utf8(get_slice(context, memory, ptr, len)?).map_err(|_| Trap::new("invalid utf-8"))
}

pub fn get_state_and_map_ptr<T>(
    context: StoreContextMut<T>,
    memory: Memory,
    ptr: u32,
) -> Result<(&mut T, *const u8), Trap> {
    // TODO: Pass in a range and check that range.
    if (ptr as usize) < memory.data_size(context.as_context()) {
        let (data, process_state) = memory.data_and_store_mut(context);
        let mapped_ptr = unsafe { data.as_ptr().offset(ptr as _) };
        Ok((process_state, mapped_ptr))
    } else {
        Err(Trap::new("pointer out of range"))
    }
}

pub fn get_state_and_map_mut_ptr<T>(
    context: StoreContextMut<T>,
    memory: Memory,
    ptr: u32,
) -> Result<(&mut T, *mut u8), Trap> {
    // TODO: Pass in a range and check that range.
    if (ptr as usize) < memory.data_size(context.as_context()) {
        let (data, process_state) = memory.data_and_store_mut(context);
        let mapped_ptr = unsafe { data.as_mut_ptr().offset(ptr as _) };
        Ok((process_state, mapped_ptr))
    } else {
        Err(Trap::new("pointer out of range"))
    }
}
