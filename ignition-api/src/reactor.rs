use alloc::collections::BTreeMap;
use alloc::format;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::Waker;

use lazy_static::lazy_static;
use spin::Mutex;

use crate::{log, TaskId};

lazy_static! {
    static ref REACTOR: Arc<Mutex<Reactor>> = Default::default();
}

pub fn alloc_task_id() -> TaskId {
    REACTOR.lock().alloc_task_id()
}

pub fn register_done_flag(task_id: TaskId, done_flag: Arc<AtomicBool>) {
    REACTOR.lock().register_done_flag(task_id, done_flag);
}

pub fn register_waker(task_id: TaskId, waker: Waker) {
    REACTOR.lock().register_waker(task_id, waker);
}

pub fn dispatch_wake(task_id: TaskId) {
    REACTOR.lock().dispatch_wake(task_id);
}

#[derive(Default)]
struct Reactor {
    next_task_id: u32,
    done_flags: BTreeMap<TaskId, Arc<AtomicBool>>,
    wakers: BTreeMap<TaskId, Waker>,
}

impl Reactor {
    pub fn alloc_task_id(&mut self) -> TaskId {
        let task_id = TaskId::new(self.next_task_id);
        // TODO: Define API semantics well enough that ID reuse is sound.
        self.next_task_id = self.next_task_id.checked_add(1).unwrap();
        task_id
    }

    pub fn register_done_flag(&mut self, task_id: TaskId, done_flag: Arc<AtomicBool>) {
        self.done_flags.insert(task_id, done_flag);
    }

    pub fn register_waker(&mut self, task_id: TaskId, waker: Waker) {
        self.wakers.insert(task_id, waker);
    }

    pub fn dispatch_wake(&mut self, task_id: TaskId) {
        // Check done flags before wakers because we don't want to wake a task only to have it go
        // right back to pending.
        if let Some(done_flag) = self.done_flags.remove(&task_id) {
            log(&format!(
                "Setting the done flag associated with {:?}",
                task_id,
            ));
            done_flag.store(true, Ordering::SeqCst)
        }

        if let Some(waker) = self.wakers.remove(&task_id) {
            log(&format!("Waking the waker associated with {:?}", task_id));
            waker.wake();
        }
    }
}
