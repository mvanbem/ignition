#![no_std]

extern crate alloc;

use crate::runtime::executor::run;
use crate::runtime::reactor::dispatch_wake;
use crate::task_id::TaskId;

pub mod api;
mod instant;
pub mod runtime;
mod task_id;

pub use crate::instant::Instant;

#[doc(hidden)]
pub fn wake_internal(task_id: u32, init: fn()) {
    let task_id = TaskId::new(task_id);
    if task_id == TaskId::INIT {
        init();
    } else {
        dispatch_wake(task_id);
    }
    run();
}

// TODO: Expand this into a fancy proc macro, something like #[ignition::init] to wrap init().
#[macro_export]
macro_rules! emit_wake {
    ($init:ident) => {
        #[no_mangle]
        pub extern "C" fn wake(task_id: u32) {
            ::ignition_guest::wake_internal(task_id, $init);
        }
    };
}
