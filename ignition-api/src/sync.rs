//! Synchronous rusty bindings for asynchronous Ignition functions. These implementations back the
//! async functions in the parent module.

use core::convert::TryFrom;
use core::time::Duration;

use crate::reactor;
use crate::sys::{self, Microseconds};
use crate::TaskId;

pub fn sleep(duration: Duration) -> TaskId {
    let task_id = reactor::new_task();
    let duration = Microseconds::try_from(duration).unwrap();

    // SAFETY: No special considerations.
    unsafe { sys::sleep(task_id.into(), duration) };

    task_id
}
