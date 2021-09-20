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
    pub fn impulse(task_id: u32);

    //
    // Time Functions
    //

    /// Asynchronously starts a timer. After it elapses, wake() will be called precisely once with
    /// the given task_id.
    pub fn sleep(task_id: u32, usec: u32);

    /// Gets the current time in microseconds according to a monotonic clock with unspecified epoch.
    pub fn monotonic_time() -> u64;
}
