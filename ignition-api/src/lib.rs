#![no_std]

extern crate alloc;

use alloc::sync::Arc;
use core::ffi::c_void;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::{Context, Poll};
use core::time::Duration;

use crate::task_id::TaskId;

pub mod executor;
mod reactor;
mod sync;
mod sys;
mod task;
mod task_id;

pub fn shutdown() {
    // SAFETY: No special considerations.
    unsafe { sys::shutdown() }
}

pub fn abort() -> ! {
    // SAFETY: No special considerations.
    unsafe { sys::abort() }
}

pub fn log(message: &str) {
    // SAFETY: `message` and `len` refer to a UTF-8 string.
    unsafe { sys::log(message.as_bytes().as_ptr() as *const c_void, message.len()) }
}

pub fn sleep(duration: Duration) -> impl Future<Output = ()> {
    let task_id = sync::sleep(duration);
    let done_flag = Arc::default();
    reactor::register_done_flag(task_id, Arc::clone(&done_flag));
    TimerFuture { task_id, done_flag }
}

struct TimerFuture {
    task_id: TaskId,
    done_flag: Arc<AtomicBool>,
}

impl Future for TimerFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
        if self.done_flag.load(Ordering::SeqCst) {
            Poll::Ready(())
        } else {
            reactor::register_waker(self.task_id, cx.waker().clone());
            Poll::Pending
        }
    }
}

#[doc(hidden)]
pub fn wake_internal(task_id: u32, init: fn()) {
    let task_id = TaskId::new(task_id);
    if task_id == TaskId::INIT {
        init();
        executor::run();
    } else {
        reactor::dispatch_wake(task_id);
        executor::run();
    }
}

// TODO: Expand this into a fancy proc macro, something like #[ignition::init] to wrap init().
#[macro_export]
macro_rules! emit_wake {
    ($init:ident) => {
        #[no_mangle]
        pub extern "C" fn wake(task_id: u32) {
            ::ignition_api::wake_internal(task_id, $init);
        }
    };
}