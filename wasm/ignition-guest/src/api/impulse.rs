use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use crate::api::sys;
use crate::runtime::reactor;
use crate::task_id::TaskId;

fn impulse_sync() -> TaskId {
    let task_id = reactor::new_task();

    // SAFETY: No special considerations.
    unsafe { sys::impulse(task_id.as_u32()) };

    task_id
}

pub fn impulse() -> impl Future<Output = ()> {
    let task_id = impulse_sync();
    Impulse { task_id }
}

struct Impulse {
    task_id: TaskId,
}

impl Drop for Impulse {
    fn drop(&mut self) {
        reactor::future_dropped(self.task_id);
    }
}

impl Future for Impulse {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
        if reactor::wake_has_happened(self.task_id) {
            Poll::Ready(())
        } else {
            reactor::store_waker(self.task_id, cx.waker().clone());
            Poll::Pending
        }
    }
}
