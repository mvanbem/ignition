use std::sync::{Arc, Mutex};
use std::task::Poll;

use wasmtime::{AsContext, AsContextMut, Caller, Trap};

use crate::interop::rpc::{RpcMetadata, RpcServerParams};
use crate::interop::{FromWasm, ToWasm, Wasm};
use crate::util::{get_memory, get_slice, get_slice_mut};
use crate::{ProcessState, TaskId};

pub fn rpc_server_create(
    mut caller: Caller<'_, Arc<Mutex<ProcessState>>>,
    params_ptr: u32,
) -> Result<u32, Trap> {
    let memory = get_memory(&mut caller)?;
    let mut params_data = get_slice(
        caller.as_context(),
        memory,
        params_ptr,
        RpcServerParams::SIZE,
    )?;

    let params = RpcServerParams::from_wasm(caller.as_context(), memory, &mut params_data)?;
    Ok(ProcessState::rpc_server_create(caller.data(), &params))
}

pub fn rpc_server_get_request(
    mut caller: Caller<'_, Arc<Mutex<ProcessState>>>,
    task_id: u32,
    rpc_server: u32,
    metadata_ptr: u32,
) -> Result<u32, Trap> {
    let memory = get_memory(&mut caller)?;

    let result = caller
        .data()
        .lock()
        .unwrap()
        .rpc_server_get_request(TaskId(task_id), rpc_server)?;
    match result {
        Poll::Ready(metadata) => {
            metadata.to_wasm(&mut get_slice_mut(
                caller.as_context_mut(),
                memory,
                metadata_ptr,
                RpcMetadata::SIZE,
            )?)?;
            Ok(0)
        }
        Poll::Pending => Ok(1),
    }
}
