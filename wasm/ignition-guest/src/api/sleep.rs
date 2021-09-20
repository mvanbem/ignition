use core::convert::TryInto;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use core::time::Duration;

use crate::api::sys;
use crate::runtime::reactor;
use crate::task_id::TaskId;

fn sleep_sync(duration: Duration) -> TaskId {
    let task_id = reactor::new_task();
    let usec = duration.as_micros().try_into().unwrap();

    // SAFETY: No special considerations.
    unsafe { sys::sleep(task_id.as_u32(), usec) };

    task_id
}

pub fn sleep(duration: Duration) -> impl Future<Output = ()> {
    let task_id = sleep_sync(duration);
    TimerFuture { task_id }
}

struct TimerFuture {
    task_id: TaskId,
}

impl Drop for TimerFuture {
    fn drop(&mut self) {
        reactor::future_dropped(self.task_id);
    }
}

impl Future for TimerFuture {
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
