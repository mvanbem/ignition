#![no_std]

extern crate alloc;

use crate::api::sys::TaskId;
use crate::runtime::executor::run;
use crate::runtime::reactor::dispatch_wake;

pub mod api;
mod instant;
pub mod io;
pub mod rpc_client;
pub mod rpc_server;
pub mod runtime;

pub use crate::instant::Instant;

#[doc(hidden)]
pub fn wake_internal(task_id: u32, param: usize, init: fn()) {
    if task_id == u32::MAX {
        init();
    } else {
        dispatch_wake(TaskId(task_id), param);
    }
    run();
}

// TODO: Expand this into a fancy proc macro, something like #[ignition::init] to wrap init().
#[macro_export]
macro_rules! emit_wake {
    ($init:ident) => {
        #[no_mangle]
        pub extern "C" fn wake(task_id: u32, param: usize) {
            ::ignition_guest::wake_internal(task_id, param, $init);
        }
    };
}
