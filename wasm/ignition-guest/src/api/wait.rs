use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use crate::api::sys::TaskId;
use crate::runtime::reactor;

pub(crate) fn wait(task_id: TaskId) -> impl Future<Output = usize> {
    Wait { task_id }
}

struct Wait {
    task_id: TaskId,
}

impl Drop for Wait {
    fn drop(&mut self) {
        reactor::future_dropped(self.task_id);
    }
}

impl Future for Wait {
    type Output = usize;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<usize> {
        if let Some(param) = reactor::get_wake_param(self.task_id) {
            Poll::Ready(param)
        } else {
            reactor::store_waker(self.task_id, cx.waker().clone());
            Poll::Pending
        }
    }
}
