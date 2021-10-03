use std::ptr::copy_nonoverlapping;
use std::sync::{Arc, Mutex};
use std::task::Poll;

use replace_with::{replace_with_or_abort, replace_with_or_abort_and_return};
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
        replace_with_or_abort(&mut self.state, |state| match state {
            PipeState::Idle => PipeState::Closed,
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
                PipeState::Closed
            }
            PipeState::PendingWrite { .. } => {
                todo!("write to a closed pipe")
            }
            PipeState::Closed => PipeState::Closed,
        });
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
        replace_with_or_abort_and_return(&mut inner.state, |state| match state {
            PipeState::Idle => (
                Poll::Pending,
                PipeState::PendingRead {
                    read_wake_queue_sender: read_wake_queue_sender.clone(),
                    read_task_id,
                    dst: SendPointerMut(dst),
                    dst_len,
                },
            ),
            PipeState::PendingRead { .. } => {
                todo!("read with a read already pending")
            }
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
                (Poll::Ready(len), PipeState::Idle)
            }
            PipeState::Closed => (Poll::Ready(0), PipeState::Closed),
        })
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
        replace_with_or_abort_and_return(&mut inner.state, |state| match state {
            PipeState::Idle => (
                Poll::Pending,
                PipeState::PendingWrite {
                    write_wake_queue_sender: write_wake_queue_sender.clone(),
                    write_task_id,
                    src: SendPointer(src),
                    src_len,
                },
            ),
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
                (Poll::Ready(len), PipeState::Idle)
            }
            PipeState::PendingWrite { .. } => todo!("write with a write already pending"),
            PipeState::Closed => todo!("write to a closed pipe"),
        })
    }

    pub fn close(&self) {
        self.inner.lock().unwrap().close();
    }
}
