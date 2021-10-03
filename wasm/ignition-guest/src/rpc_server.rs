use std::future::Future;
use std::mem::MaybeUninit;
use std::pin::Pin;

use crate::api::log;
use crate::api::sys::{self, RpcMethodMetadata, RpcServerMethod, RpcServerParams};
use crate::api::wait::wait;
use crate::io::{ReadHandle, WriteHandle};
use crate::runtime::reactor::new_task;
use crate::runtime::spawn;

pub type RpcFuture = Pin<Box<dyn Future<Output = ()> + Send + Sync>>;
pub type Handler = Box<dyn Fn(ReadHandle, WriteHandle) -> RpcFuture + Send + Sync>;

pub struct RpcServerBuilder {
    name: String,
    methods: Vec<MethodBuilder>,
}

struct MethodBuilder {
    name: String,
    handler: Handler,
}

impl RpcServerBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            methods: Default::default(),
        }
    }

    pub fn add_handler(mut self, name: &str, handler: Handler) -> Self {
        self.methods.push(MethodBuilder {
            name: name.to_owned(),
            handler,
        });
        self
    }

    pub fn build(self) {
        let methods: Vec<_> = self
            .methods
            .iter()
            .map(|method| RpcServerMethod {
                method_name_ptr: method.name.as_ptr(),
                method_name_len: method.name.len(),
            })
            .collect();

        // SAFETY: No special considerations.
        let rpc_server = unsafe {
            sys::rpc_server_create(&RpcServerParams {
                service_name_ptr: self.name.as_ptr(),
                service_name_len: self.name.len(),
                methods_ptr: methods.as_ptr(),
                methods_len: methods.len(),
            })
        };

        let handlers: Vec<Handler> = self
            .methods
            .into_iter()
            .map(|method| method.handler)
            .collect();
        spawn(async move {
            loop {
                let task_id = new_task();
                loop {
                    let mut metadata: MaybeUninit<RpcMethodMetadata> = MaybeUninit::uninit();

                    // SAFETY: `metadata` points to an appropriately sized space.
                    let result = unsafe {
                        sys::rpc_server_get_request(task_id, rpc_server, metadata.as_mut_ptr())
                    };
                    if result.0 != 0 {
                        break;
                    }

                    // SAFETY: rpc_server_handle() initializes the metadata.
                    let metadata = unsafe { metadata.assume_init() };

                    log(&format!(
                        "incoming RPC: method={}, request_io={}, response_io={}",
                        metadata.index, metadata.request_io.0, metadata.response_io.0,
                    ));
                    let request = ReadHandle::from_raw(metadata.request_io);
                    let response = WriteHandle::from_raw(metadata.response_io);

                    let handler = &*handlers[metadata.index as usize];
                    handler(request, response).await;
                }
                wait(task_id).await;
            }
        });
    }
}
