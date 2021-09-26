use std::sync::{Arc, Mutex};
use std::task::Poll;

use byteorder::{LittleEndian, WriteBytesExt};
use wasmtime::{AsContextMut, Caller, Trap};

use crate::util::{get_memory, get_slice_mut, get_state_and_map_mut_ptr, get_state_and_map_ptr};
use crate::{ProcessState, TaskId};

pub fn io_read(
    mut caller: Caller<'_, Arc<Mutex<ProcessState>>>,
    task_id: u32,
    io: u32,
    ptr: u32,
    len: u32,
    n_ptr: u32,
) -> Result<u32, Trap> {
    let memory = get_memory(&mut caller)?;
    let (process_state, dst) = get_state_and_map_mut_ptr(caller.as_context_mut(), memory, ptr)?;
    let result = process_state
        .lock()
        .unwrap()
        .io_read(TaskId(task_id), io as _, dst, len)?;
    match result {
        Poll::Ready(n) => {
            let mut n_data = get_slice_mut(caller.as_context_mut(), memory, n_ptr, 4)?;
            n_data.write_u32::<LittleEndian>(n).unwrap();
            Ok(0)
        }
        Poll::Pending => Ok(1),
    }
}

pub fn io_write(
    mut caller: Caller<'_, Arc<Mutex<ProcessState>>>,
    task_id: u32,
    io: u32,
    ptr: u32,
    len: u32,
    n_ptr: u32,
) -> Result<u32, Trap> {
    let memory = get_memory(&mut caller)?;
    let (process_state, src) = get_state_and_map_ptr(caller.as_context_mut(), memory, ptr)?;
    let result = process_state
        .lock()
        .unwrap()
        .io_write(TaskId(task_id), io, src, len)?;
    match result {
        Poll::Ready(n) => {
            let mut n_data = get_slice_mut(caller.as_context_mut(), memory, n_ptr, 4)?;
            n_data.write_u32::<LittleEndian>(n).unwrap();
            Ok(0)
        }
        Poll::Pending => Ok(1),
    }
}

pub fn io_close(caller: Caller<'_, Arc<Mutex<ProcessState>>>, io: u32) -> Result<(), Trap> {
    caller.data().lock().unwrap().io_close(io)
}
