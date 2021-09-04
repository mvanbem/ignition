use alloc::collections::{BTreeMap, VecDeque};
use alloc::sync::Arc;
use core::future::Future;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

use lazy_static::lazy_static;
use spin::Mutex;

use crate::task::Task;

lazy_static! {
    static ref EXECUTOR: Arc<Mutex<Executor>> = Default::default();
}

pub(crate) fn run() {
    executor_run(&EXECUTOR);
}

pub fn spawn(future: impl Future<Output = ()> + Send + 'static) {
    EXECUTOR.lock().spawn(Task::new(future));
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct WakeId(usize);

#[derive(Default)]
struct Executor {
    awake: VecDeque<Task>,
    asleep: BTreeMap<WakeId, Task>,
    next_wake_id: WakeId,
}

fn executor_run(arc: &Arc<Mutex<Executor>>) {
    loop {
        let mut inner = arc.lock();
        let mut task = match inner.awake.pop_front() {
            Some(task) => task,
            None => break,
        };
        let wake_id = inner.next_wake_id;
        // TODO: Make WakeIds reusable.
        inner.next_wake_id = WakeId(inner.next_wake_id.0.checked_add(1).unwrap());
        drop(inner);

        let waker = waker(wake_id);
        let mut context = Context::from_waker(&waker);
        match task.poll(&mut context) {
            Poll::Ready(()) => (),
            Poll::Pending => {
                arc.lock().asleep.insert(wake_id, task);
            }
        }
    }
}

impl Executor {
    fn spawn(&mut self, task: Task) {
        self.awake.push_back(task);
    }

    fn wake(&mut self, wake_id: WakeId) {
        if let Some(task) = self.asleep.remove(&wake_id) {
            self.spawn(task);
        }
    }
}

fn raw_waker(wake_id: WakeId) -> RawWaker {
    fn clone(ptr: *const ()) -> RawWaker {
        let wake_id = WakeId(ptr as usize);
        raw_waker(wake_id)
    }
    fn wake(ptr: *const ()) {
        let wake_id = WakeId(ptr as usize);
        EXECUTOR.lock().wake(wake_id);
    }
    fn wake_by_ref(ptr: *const ()) {
        wake(ptr);
    }
    fn drop(_ptr: *const ()) {
        // No cleanup required.
    }

    let vtable = &RawWakerVTable::new(clone, wake, wake_by_ref, drop);
    RawWaker::new(wake_id.0 as *const (), vtable)
}

fn waker(wake_id: WakeId) -> Waker {
    // SAFETY: This RawWaker follows the contract defined in RawWaker's and RawWakerVTable's
    // documentation.
    unsafe { Waker::from_raw(raw_waker(wake_id)) }
}
