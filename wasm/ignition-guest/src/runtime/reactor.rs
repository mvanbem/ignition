use std::convert::TryInto;
use std::sync::Arc;
use std::sync::Mutex;
use std::task::Waker;

use lazy_static::lazy_static;
use slab::Slab;

use crate::api::sys::TaskId;

lazy_static! {
    static ref REACTOR: Arc<Mutex<Reactor>> = Default::default();
}

pub fn new_task() -> TaskId {
    REACTOR.lock().unwrap().new_task()
}

pub fn drop_unused_task(task_id: TaskId) {
    REACTOR.lock().unwrap().drop_unused_task(task_id);
}

pub fn future_dropped(task_id: TaskId) {
    REACTOR.lock().unwrap().future_dropped(task_id);
}

pub fn store_waker(task_id: TaskId, waker: Waker) {
    REACTOR.lock().unwrap().store_waker(task_id, waker);
}

pub fn dispatch_wake(task_id: TaskId, param: usize) {
    REACTOR.lock().unwrap().dispatch_wake(task_id, param);
}

pub fn get_wake_param(task_id: TaskId) -> Option<usize> {
    REACTOR.lock().unwrap().get_wake_param(task_id)
}

#[derive(Default)]
struct Reactor {
    tasks: Slab<TaskState>,
}

struct TaskState {
    waker: Option<Waker>,
    // None before wakened and Some after.
    wake_param: Option<usize>,
    future_is_dropped: bool,
}

impl TaskState {
    fn new() -> Self {
        Self {
            waker: None,
            wake_param: None,
            future_is_dropped: false,
        }
    }

    fn ready_to_drop(&self) -> bool {
        self.wake_param.is_some() && self.future_is_dropped
    }
}

impl Reactor {
    fn new_task(&mut self) -> TaskId {
        TaskId(self.tasks.insert(TaskState::new()).try_into().unwrap())
    }

    fn drop_unused_task(&mut self, task_id: TaskId) {
        self.tasks.remove(task_id.0 as usize);
    }

    fn future_dropped(&mut self, task_id: TaskId) {
        let task_state = self.tasks.get_mut(task_id.0 as usize).unwrap();
        task_state.future_is_dropped = true;

        if task_state.ready_to_drop() {
            self.drop_task_state(task_id);
        }
    }

    fn drop_task_state(&mut self, task_id: TaskId) {
        self.tasks.remove(task_id.0 as usize);
    }

    fn store_waker(&mut self, task_id: TaskId, waker: Waker) {
        let task_state = self.tasks.get_mut(task_id.0 as usize).unwrap();
        task_state.waker = Some(waker);
    }

    fn dispatch_wake(&mut self, task_id: TaskId, param: usize) {
        let task_state = self.tasks.get_mut(task_id.0 as usize).unwrap();
        task_state.wake_param = Some(param);
        if let Some(waker) = task_state.waker.take() {
            waker.wake();
        }

        if task_state.ready_to_drop() {
            self.drop_task_state(task_id);
        }
    }

    fn get_wake_param(&self, task_id: TaskId) -> Option<usize> {
        self.tasks.get(task_id.0 as usize).unwrap().wake_param
    }
}
