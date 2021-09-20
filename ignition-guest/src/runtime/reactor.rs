use alloc::sync::Arc;
use core::task::Waker;

use lazy_static::lazy_static;
use spin::Mutex;

use crate::runtime::free_list::FreeList;
use crate::task_id::TaskId;

lazy_static! {
    static ref REACTOR: Arc<Mutex<Reactor>> = Default::default();
}

pub fn new_task() -> TaskId {
    REACTOR.lock().new_task()
}

pub fn future_dropped(task_id: TaskId) {
    REACTOR.lock().future_dropped(task_id);
}

pub fn store_waker(task_id: TaskId, waker: Waker) {
    REACTOR.lock().store_waker(task_id, waker);
}

pub fn dispatch_wake(task_id: TaskId) {
    REACTOR.lock().dispatch_wake(task_id);
}

pub fn wake_has_happened(task_id: TaskId) -> bool {
    REACTOR.lock().wake_has_happened(task_id)
}

#[derive(Default)]
struct Reactor {
    tasks: FreeList<TaskId, TaskState>,
}

#[derive(Default)]
struct TaskState {
    waker: Option<Waker>,
    wake_has_happened: bool,
    future_is_dropped: bool,
}

impl TaskState {
    fn ready_to_drop(&self) -> bool {
        self.wake_has_happened && self.future_is_dropped
    }
}

impl Reactor {
    fn new_task(&mut self) -> TaskId {
        self.tasks.insert(TaskState::default())
    }

    fn future_dropped(&mut self, task_id: TaskId) {
        let task_state = self.tasks.get_mut(task_id);
        task_state.future_is_dropped = true;

        if task_state.ready_to_drop() {
            self.drop_task_state(task_id);
        }
    }

    fn drop_task_state(&mut self, task_id: TaskId) {
        self.tasks.remove(task_id);
    }

    fn store_waker(&mut self, task_id: TaskId, waker: Waker) {
        let task_state = self.tasks.get_mut(task_id);
        task_state.waker = Some(waker);
    }

    fn dispatch_wake(&mut self, task_id: TaskId) {
        let task_state = self.tasks.get_mut(task_id);
        task_state.wake_has_happened = true;
        if let Some(waker) = task_state.waker.take() {
            waker.wake();
        }

        if task_state.ready_to_drop() {
            self.drop_task_state(task_id);
        }
    }

    fn wake_has_happened(&self, task_id: TaskId) -> bool {
        self.tasks.get(task_id).wake_has_happened
    }
}
