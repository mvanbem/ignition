use std::sync::Arc;
use std::task::Poll;

use byteorder::{LittleEndian, WriteBytesExt};
use wasmtime::{AsContext, AsContextMut, Caller, Trap};

use crate::util::{get_memory, get_slice_mut, get_str};
use crate::{Process, TaskId};

pub fn rpc_client_create(
    mut caller: Caller<'_, Arc<Process>>,
    service_name_ptr: u32,
    service_name_len: u32,
) -> Result<u32, Trap> {
    let memory = get_memory(&mut caller)?;
    let service_name = get_str(
        caller.as_context(),
        memory,
        service_name_ptr,
        service_name_len,
    )?
    .to_owned();

    Ok(caller.data().rpc_client_create(service_name))
}

pub fn rpc_client_wait_healthy(
    caller: Caller<'_, Arc<Process>>,
    task_id: u32,
    rpc_client: u32,
) -> Result<u32, Trap> {
    match Process::rpc_client_wait_healthy(caller.data(), TaskId(task_id), rpc_client)? {
        Poll::Ready(()) => Ok(0),
        Poll::Pending => Ok(1),
    }
}

pub fn rpc_client_request(
    mut caller: Caller<'_, Arc<Process>>,
    rpc_client: u32,
    method_name_ptr: u32,
    method_name_len: u32,
    request_io_ptr: u32,
    response_io_ptr: u32,
) -> Result<u32, Trap> {
    let memory = get_memory(&mut caller)?;
    let method_name = get_str(
        caller.as_context(),
        memory,
        method_name_ptr,
        method_name_len,
    )?
    .to_owned();

    let (request_io, response_io) =
        Process::rpc_client_request(caller.data(), rpc_client, &method_name)?;

    let mut request_io_data = get_slice_mut(caller.as_context_mut(), memory, request_io_ptr, 4)?;
    request_io_data
        .write_u32::<LittleEndian>(request_io)
        .unwrap();

    let mut response_io_data = get_slice_mut(caller.as_context_mut(), memory, response_io_ptr, 4)?;
    response_io_data
        .write_u32::<LittleEndian>(response_io)
        .unwrap();

    Ok(0)
}
