use alloc::sync::Arc;
use alloc::vec::Vec;
use core::convert::TryInto;
use core::task::Waker;

use lazy_static::lazy_static;
use spin::Mutex;

use crate::TaskId;

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
    nodes: Vec<Node>,
    first_free_task_id: Option<TaskId>,
}

enum Node {
    Free { next_free_task_id: Option<TaskId> },
    Allocated(TaskState),
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
    pub fn new_task(&mut self) -> TaskId {
        if let Some(task_id) = self.first_free_task_id {
            // At least one free node exists.

            // Remove the free node at the head of the free list by replacing the free list with its
            // tail.
            self.first_free_task_id = match self.nodes[task_id.as_usize()] {
                Node::Free { next_free_task_id } => next_free_task_id,
                _ => panic!("bad free list"),
            };

            // Overwrite the allocated node.
            self.nodes[task_id.as_usize()] = Node::Allocated(TaskState::default());

            task_id
        } else {
            // There are no free nodes.

            // Add a new one.
            self.nodes.push(Node::Allocated(TaskState::default()));

            TaskId::new((self.nodes.len() - 1).try_into().unwrap())
        }
    }

    pub fn future_dropped(&mut self, task_id: TaskId) {
        let task_state = match &mut self.nodes[task_id.as_usize()] {
            Node::Allocated(task_state) => task_state,
            _ => panic!("future_dropped() called on a free node"),
        };
        task_state.future_is_dropped = true;

        if task_state.ready_to_drop() {
            self.drop_task_state(task_id);
        }
    }

    fn drop_task_state(&mut self, task_id: TaskId) {
        // This function is only called internally, so assume the node is known to be allocated.

        // Overwrite this node and push it onto the free list.
        self.nodes[task_id.as_usize()] = Node::Free {
            next_free_task_id: self.first_free_task_id,
        };
        self.first_free_task_id = Some(task_id);
    }

    pub fn store_waker(&mut self, task_id: TaskId, waker: Waker) {
        let task_state = match &mut self.nodes[task_id.as_usize()] {
            Node::Allocated(task_state) => task_state,
            _ => panic!("store_waker() called on a free node"),
        };
        task_state.waker = Some(waker);
    }

    pub fn dispatch_wake(&mut self, task_id: TaskId) {
        let task_state = match &mut self.nodes[task_id.as_usize()] {
            Node::Allocated(task_state) => task_state,
            _ => panic!("dispatch_wake() called on a free node"),
        };

        task_state.wake_has_happened = true;
        if let Some(waker) = task_state.waker.take() {
            waker.wake();
        }

        if task_state.ready_to_drop() {
            self.drop_task_state(task_id);
        }
    }

    pub fn wake_has_happened(&self, task_id: TaskId) -> bool {
        let task_state = match &self.nodes[task_id.as_usize()] {
            Node::Allocated(task_state) => task_state,
            _ => panic!("wake_has_happened() called on a free node"),
        };
        task_state.wake_has_happened
    }
}
