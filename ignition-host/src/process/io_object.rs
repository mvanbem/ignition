use std::task::Poll;

use tokio::sync::mpsc::UnboundedSender;

use crate::process::pipe::{PipeReader, PipeWriter};
use crate::{TaskId, WakeParams};

pub struct IoObject {
    reader: Option<PipeReader>,
    writer: Option<PipeWriter>,
}

impl IoObject {
    pub fn new_reader(reader: PipeReader) -> Self {
        Self {
            reader: Some(reader),
            writer: None,
        }
    }

    pub fn new_writer(writer: PipeWriter) -> Self {
        Self {
            reader: None,
            writer: Some(writer),
        }
    }

    pub unsafe fn read(
        &mut self,
        wake_queue_sender: &UnboundedSender<WakeParams>,
        task_id: TaskId,
        dst: *mut u8,
        len: u32,
    ) -> Poll<u32> {
        unsafe {
            self.reader
                .as_mut()
                .unwrap()
                .read(wake_queue_sender, task_id, dst, len)
        }
    }

    pub unsafe fn write(
        &mut self,
        wake_queue_sender: &UnboundedSender<WakeParams>,
        task_id: TaskId,
        src: *const u8,
        len: u32,
    ) -> Poll<u32> {
        unsafe {
            self.writer
                .as_mut()
                .unwrap()
                .write(wake_queue_sender, task_id, src, len)
        }
    }

    pub fn close(self) {
        if let Some(reader) = self.reader {
            reader.close();
        }
        if let Some(writer) = self.writer {
            writer.close();
        }
    }
}
