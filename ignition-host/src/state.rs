use std::sync::atomic::{AtomicBool, Ordering};

use tokio::sync::mpsc::{channel, Receiver, Sender};

use crate::TaskId;

pub struct State {
    is_shutdown: AtomicBool,
    wake_queue_sender: Sender<TaskId>,
}

impl State {
    pub fn new() -> (Self, Receiver<TaskId>) {
        let (wake_queue_sender, wake_queue_receiver) = channel(1);
        let state = State {
            is_shutdown: AtomicBool::new(false),
            wake_queue_sender,
        };
        (state, wake_queue_receiver)
    }

    pub fn shutdown(&self) {
        // TODO: Relax ordering?
        self.is_shutdown.store(true, Ordering::SeqCst);
    }

    pub fn is_shutdown(&self) -> bool {
        // TODO: Relax ordering?
        self.is_shutdown.load(Ordering::SeqCst)
    }

    pub fn wake_queue_sender(&self) -> &Sender<TaskId> {
        &self.wake_queue_sender
    }
}
