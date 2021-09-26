//! Bindings for the Ignition C API.

use core::ffi::c_void;

#[link(wasm_import_module = "ignition")]
extern "C" {
    //
    // Core Functions
    //

    /// Requests that this instance be destroyed after the current wake() invocation returns.
    pub fn shutdown();

    /// Immediately ends execution and destroys this instance.
    pub fn abort() -> !;

    //
    // Debug, test, and diagnostic functions.
    //

    /// Emits a debug log message.
    pub fn log(ptr: *const c_void, len: usize);

    /// Requests that wake() be called precisely once with the given task_id.
    pub fn impulse(task_id: TaskId);

    //
    // Time Functions
    //

    /// Asynchronously starts a timer. After it elapses, wake() will be called precisely once with
    /// the given task_id.
    pub fn sleep(task_id: TaskId, usec: u32);

    /// Gets the current time in microseconds according to a monotonic clock with unspecified epoch.
    pub fn monotonic_time() -> u64;

    //
    // I/O Functions
    //

    pub fn io_read(
        task_id: TaskId,
        io: IoHandle,
        ptr: *mut u8,
        len: usize,
        n_ptr: *mut usize,
    ) -> u32;

    pub fn io_write(
        task_id: TaskId,
        io: IoHandle,
        ptr: *const u8,
        len: usize,
        n_ptr: *mut usize,
    ) -> u32;

    pub fn io_close(io: IoHandle);

    //
    // RPC Client Functions
    //

    pub fn rpc_client_create(
        service_name_ptr: *const u8,
        service_name_len: usize,
    ) -> RpcClientHandle;

    pub fn rpc_client_wait_healthy(task_id: TaskId, rpc_client: RpcClientHandle) -> u32;

    pub fn rpc_client_request(
        rpc_client: RpcClientHandle,
        method_name_ptr: *const u8,
        method_name_len: usize,
        request_io_ptr: *mut IoHandle,
        response_io_ptr: *mut IoHandle,
    ) -> u32;

    //
    // RPC Server Functions
    //

    pub fn rpc_server_create(params: *const RpcServerParams) -> RpcServerHandle;

    pub fn rpc_server_get_request(
        task_id: TaskId,
        rpc_server: RpcServerHandle,
        metadata: *mut RpcMethodMetadata,
    ) -> RpcServerGetRequestResult;
}

#[must_use]
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct RpcServerGetRequestResult(pub u32);

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct TaskId(pub u32);

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct IoHandle(pub u32);

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct RpcServerHandle(pub u32);

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct RpcClientHandle(pub u32);

#[repr(C)]
pub struct RpcServerParams {
    pub service_name_ptr: *const u8,
    pub service_name_len: usize,
    pub methods_ptr: *const RpcServerMethod,
    pub methods_len: usize,
}

#[repr(C)]
pub struct RpcServerMethod {
    pub method_name_ptr: *const u8,
    pub method_name_len: usize,
}

#[repr(C)]
pub struct RpcMethodMetadata {
    pub index: usize,
    pub request_io: IoHandle,
    pub response_io: IoHandle,
}
