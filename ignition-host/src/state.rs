use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use tokio::sync::mpsc::{channel, Receiver, Sender};

use crate::TaskId;

pub struct State {
    is_shutdown: AtomicBool,
    impulse_queue: VecDeque<TaskId>,
    wake_queue_sender: Sender<TaskId>,
    start_time: Instant,
}

impl State {
    pub fn new() -> (Self, Receiver<TaskId>) {
        let (wake_queue_sender, wake_queue_receiver) = channel(1);
        let state = State {
            is_shutdown: AtomicBool::new(false),
            impulse_queue: VecDeque::new(),
            wake_queue_sender,
            start_time: Instant::now(),
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

    pub fn impulse_queue_mut(&mut self) -> &mut VecDeque<TaskId> {
        &mut self.impulse_queue
    }

    pub fn wake_queue_sender(&self) -> &Sender<TaskId> {
        &self.wake_queue_sender
    }

    pub fn start_time(&self) -> Instant {
        self.start_time
    }
}
