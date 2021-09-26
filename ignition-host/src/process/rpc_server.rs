use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::task::Poll;

use tokio::sync::mpsc::UnboundedSender;

use crate::interop::rpc::RpcMetadata;
use crate::{TaskId, WakeParams};

pub struct RpcServer {
    waiting_task_ids: HashSet<TaskId>,
    method_index_by_name: HashMap<String, u32>,
    wake_queue_sender: UnboundedSender<WakeParams>,
    request_queue: Vec<RpcMetadata>,
}

impl RpcServer {
    pub fn new(method_names: Vec<String>, wake_queue_sender: UnboundedSender<WakeParams>) -> Self {
        Self {
            waiting_task_ids: HashSet::new(),
            method_index_by_name: method_names
                .into_iter()
                .enumerate()
                .map(|(index, method_name)| (method_name, index.try_into().unwrap()))
                .collect(),
            wake_queue_sender,
            request_queue: Vec::new(),
        }
    }

    pub fn get_request(&mut self, task_id: TaskId) -> Poll<RpcMetadata> {
        if let Some(request) = self.request_queue.pop() {
            Poll::Ready(request)
        } else {
            self.waiting_task_ids.insert(task_id);
            Poll::Pending
        }
    }

    pub fn queue_request(&mut self, method_name: &str, request_io: u32, response_io: u32) {
        let method_index = self.method_index_by_name[method_name];

        self.request_queue.push(RpcMetadata {
            method_index,
            request_io,
            response_io,
        });

        for task_id in self.waiting_task_ids.drain() {
            self.wake_queue_sender
                .send(WakeParams { task_id, param: 0 })
                .unwrap();
        }
    }
}
