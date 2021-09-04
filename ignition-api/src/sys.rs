//! Bindings for the Ignition C API.

use core::convert::{TryFrom, TryInto};
use core::ffi::c_void;
use core::num::TryFromIntError;
use core::time::Duration;

#[repr(transparent)]
pub struct TaskId(pub u32);

#[repr(transparent)]
pub struct Microseconds(pub u32);

impl TryFrom<Duration> for Microseconds {
    type Error = TryFromIntError;

    fn try_from(duration: Duration) -> Result<Self, Self::Error> {
        Ok(Microseconds(duration.as_micros().try_into()?))
    }
}

#[link(wasm_import_module = "ignition")]
extern "C" {
    //
    // Core Functions
    //

    /// Requests that this instance be destroyed after the current wake() invocation returns.
    pub fn shutdown();

    /// Immediately ends execution and destroys this instance.
    pub fn abort() -> !;

    /// Emits a debug log message.
    pub fn log(ptr: *const c_void, len: usize);

    //
    // Time Functions
    //

    /// Asynchronously starts a timer. After it elapses, wake() will be called precisely once with
    /// the given task_id.
    pub fn sleep(task_id: TaskId, duration: Microseconds);
}
