use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::task::Poll;

use lazy_static::lazy_static;

use crate::process::process::Process;
use crate::util::pointer_identity_arc::PointerIdentityArc;
use crate::{TaskId, WakeParams};

#[derive(Default)]
pub struct ServiceRegistry {
    inner: Mutex<InnerServiceRegistry>,
}

#[derive(Default)]
struct InnerServiceRegistry {
    servers_by_service_name: HashMap<String, HashSet<RpcServerRef>>,
    tasks_waiting_by_service_name: HashMap<String, HashSet<ProcessTask>>,
}

impl ServiceRegistry {
    pub fn register(&self, service_name: String, rpc_server_ref: RpcServerRef) {
        let mut inner = self.inner.lock().unwrap();

        // Wake any processes that were waiting for this service to become available.
        if let Some(mut process_tasks) = inner.tasks_waiting_by_service_name.remove(&service_name) {
            for entry in process_tasks.drain() {
                // NOTE: This will be a recursive acquire if the process that's registering this
                // server had a client waiting on that same service name. That could be fixed by
                // putting an async queue in here.
                entry
                    .process
                    .wake_queue_sender()
                    .send(WakeParams {
                        task_id: entry.task_id,
                        param: 0,
                    })
                    .unwrap();
            }
        }

        // Add this server to the registry.
        inner
            .servers_by_service_name
            .entry(service_name)
            .or_default()
            .insert(rpc_server_ref);
    }

    pub fn wait_for_server(
        &self,
        process: &Arc<Process>,
        task_id: TaskId,
        service_name: Cow<str>,
    ) -> Poll<()> {
        let mut inner = self.inner.lock().unwrap();

        if inner.servers_by_service_name.contains_key(&*service_name) {
            Poll::Ready(())
        } else {
            inner
                .tasks_waiting_by_service_name
                .entry(service_name.into_owned())
                .or_default()
                .insert(ProcessTask {
                    process: PointerIdentityArc::new(Arc::clone(process)),
                    task_id,
                });
            Poll::Pending
        }
    }

    pub fn pick_server_or_die(&self, service_name: &str) -> RpcServerRef {
        self.inner
            .lock()
            .unwrap()
            .servers_by_service_name
            .get(service_name)
            .and_then(|set| set.iter().next())
            .unwrap_or_else(|| panic!("RPC service {:?} has no servers", service_name))
            .clone()
    }
}

#[derive(Hash, PartialEq, Eq)]
struct ProcessTask {
    process: PointerIdentityArc<Process>,
    task_id: TaskId,
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct RpcServerRef {
    pub process: PointerIdentityArc<Process>,
    pub rpc_server: u32,
}

lazy_static! {
    pub static ref SERVICE_REGISTRY: Arc<ServiceRegistry> = Default::default();
}
