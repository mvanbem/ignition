use std::mem::MaybeUninit;
use std::ops::Deref;

use crate::api::sys::{self, IoHandle, RpcClientHandle};
use crate::api::wait::wait;
use crate::io::{ReadHandle, WriteHandle};
use crate::runtime::reactor::{drop_unused_task, new_task};

pub struct RpcClient {
    rpc_client: RpcClientHandle,
}

impl RpcClient {
    pub fn new(service_name: &str) -> Self {
        let rpc_client =
            unsafe { sys::rpc_client_create(service_name.as_ptr(), service_name.len()) };

        Self { rpc_client }
    }

    pub async fn wait_healthy(&self) {
        let task_id = new_task();
        let result = unsafe { sys::rpc_client_wait_healthy(task_id, self.rpc_client) };
        if result == 0 {
            drop_unused_task(task_id);
        } else {
            wait(task_id).await;
        }
    }

    pub fn request(&self, method_name: &str) -> Request {
        let mut request_io: MaybeUninit<IoHandle> = MaybeUninit::uninit();
        let mut response_io: MaybeUninit<IoHandle> = MaybeUninit::uninit();

        let result = unsafe {
            sys::rpc_client_request(
                self.rpc_client,
                method_name.as_ptr(),
                method_name.len(),
                request_io.as_mut_ptr(),
                response_io.as_mut_ptr(),
            )
        };

        assert_eq!(result, 0);
        let request_io = unsafe { request_io.assume_init() };
        let response_io = unsafe { response_io.assume_init() };
        Request {
            request: WriteHandle::from_raw(request_io),
            response: ReadHandle::from_raw(response_io),
        }
    }
}

pub struct Request {
    request: WriteHandle,
    response: ReadHandle,
}

impl Request {
    pub fn into_response(self) -> ReadHandle {
        self.response
    }
}

impl Deref for Request {
    type Target = WriteHandle;

    fn deref(&self) -> &WriteHandle {
        &self.request
    }
}
