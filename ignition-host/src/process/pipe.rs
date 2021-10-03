use std::mem::replace;
use std::ptr::copy_nonoverlapping;
use std::sync::{Arc, Mutex};
use std::task::Poll;

use tokio::sync::mpsc::UnboundedSender;

use crate::{TaskId, WakeParams};

struct SendPointer<T>(*const T);

unsafe impl<T> Send for SendPointer<T> {}

struct SendPointerMut<T>(*mut T);

unsafe impl<T> Send for SendPointerMut<T> {}

pub struct PipeReader {
    inner: Arc<Mutex<InnerPipe>>,
}

pub struct PipeWriter {
    inner: Arc<Mutex<InnerPipe>>,
}

struct InnerPipe {
    state: PipeState,
}

enum PipeState {
    Idle,
    PendingRead {
        read_wake_queue_sender: UnboundedSender<WakeParams>,
        read_task_id: TaskId,
        dst: SendPointerMut<u8>,
        dst_len: u32,
    },
    PendingWrite {
        write_wake_queue_sender: UnboundedSender<WakeParams>,
        write_task_id: TaskId,
        src: SendPointer<u8>,
        src_len: u32,
    },
    Closed,
    Unreachable,
}

pub fn pipe() -> (PipeReader, PipeWriter) {
    let inner = Arc::new(Mutex::new(InnerPipe {
        state: PipeState::Idle,
    }));
    (
        PipeReader {
            inner: Arc::clone(&inner),
        },
        PipeWriter { inner },
    )
}

impl InnerPipe {
    fn close(&mut self) {
        match replace(&mut self.state, PipeState::Unreachable) {
            PipeState::Idle => self.state = PipeState::Closed,
            PipeState::PendingRead {
                read_wake_queue_sender,
                read_task_id,
                ..
            } => {
                read_wake_queue_sender
                    .send(WakeParams {
                        task_id: read_task_id,
                        param: 0,
                    })
                    .unwrap();
                self.state = PipeState::Closed;
            }
            PipeState::PendingWrite { .. } => {
                todo!("haven't implemented gracefully writing to a closed pipe")
            }
            PipeState::Closed => self.state = PipeState::Closed,
            PipeState::Unreachable => unreachable!(),
        }
    }
}

impl PipeReader {
    pub unsafe fn read(
        &self,
        read_wake_queue_sender: &UnboundedSender<WakeParams>,
        read_task_id: TaskId,
        dst: *mut u8,
        dst_len: u32,
    ) -> Poll<u32> {
        if dst_len == 0 {
            return Poll::Ready(0);
        }

        let mut inner = self.inner.lock().unwrap();
        match replace(&mut inner.state, PipeState::Unreachable) {
            PipeState::Idle => {
                inner.state = PipeState::PendingRead {
                    read_wake_queue_sender: read_wake_queue_sender.clone(),
                    read_task_id,
                    dst: SendPointerMut(dst),
                    dst_len,
                };
                Poll::Pending
            }
            PipeState::PendingRead { .. } => panic!(),
            PipeState::PendingWrite {
                write_wake_queue_sender,
                write_task_id,
                src,
                src_len,
            } => {
                // TODO: Potentially keep the pipe in the PendingWrite state if not all of the
                // available data was read.

                let len = dst_len.min(src_len);
                unsafe { copy_nonoverlapping(src.0, dst, len as _) }
                write_wake_queue_sender
                    .send(WakeParams {
                        task_id: write_task_id,
                        param: len,
                    })
                    .unwrap();
                inner.state = PipeState::Idle;
                Poll::Ready(len)
            }
            PipeState::Closed => {
                inner.state = PipeState::Closed;
                Poll::Ready(0)
            }
            PipeState::Unreachable => unreachable!(),
        }
    }

    pub fn close(&self) {
        self.inner.lock().unwrap().close();
    }
}

impl PipeWriter {
    pub unsafe fn write(
        &self,
        write_wake_queue_sender: &UnboundedSender<WakeParams>,
        write_task_id: TaskId,
        src: *const u8,
        src_len: u32,
    ) -> Poll<u32> {
        if src_len == 0 {
            return Poll::Ready(0);
        }

        let mut inner = self.inner.lock().unwrap();
        match replace(&mut inner.state, PipeState::Unreachable) {
            PipeState::Idle => {
                inner.state = PipeState::PendingWrite {
                    write_wake_queue_sender: write_wake_queue_sender.clone(),
                    write_task_id,
                    src: SendPointer(src),
                    src_len,
                };
                Poll::Pending
            }
            PipeState::PendingRead {
                read_wake_queue_sender,
                read_task_id,
                dst,
                dst_len,
            } => {
                let len = src_len.min(dst_len);
                unsafe { copy_nonoverlapping(src, dst.0, len as _) }
                read_wake_queue_sender
                    .send(WakeParams {
                        task_id: read_task_id,
                        param: len,
                    })
                    .unwrap();
                inner.state = PipeState::Idle;
                Poll::Ready(len)
            }
            PipeState::PendingWrite { .. } => panic!(),
            PipeState::Closed => {
                todo!("haven't implemented gracefully writing to a closed pipe")
            }
            PipeState::Unreachable => unreachable!(),
        }
    }

    pub fn close(&self) {
        self.inner.lock().unwrap().close();
    }
}
