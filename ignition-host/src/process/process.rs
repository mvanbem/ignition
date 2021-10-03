use std::borrow::Cow;
use std::convert::TryInto;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::task::Poll;
use std::time::Instant;

use slab::Slab;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use wasmtime::Trap;

use crate::interop::rpc::{RpcMetadata, RpcServerParams};
use crate::process::io_object::IoObject;
use crate::process::pipe::pipe;
use crate::process::rpc_client::RpcClient;
use crate::process::rpc_server::RpcServer;
use crate::process::service_registry::{RpcServerRef, SERVICE_REGISTRY};
use crate::util::pointer_identity_arc::PointerIdentityArc;
use crate::{TaskId, WakeParams};

pub struct Process {
    pid: usize,
    start_time: Instant,
    is_shutdown: AtomicBool,
    wake_queue_sender: UnboundedSender<WakeParams>,
    inner: Mutex<InnerProcess>,
}

struct InnerProcess {
    rpc_clients: Slab<RpcClient>,
    rpc_servers: Slab<RpcServer>,
    io_objects: Slab<IoObject>,
}

impl Process {
    pub fn new(pid: usize) -> (Self, UnboundedReceiver<WakeParams>) {
        let (wake_queue_sender, wake_queue_receiver) = unbounded_channel();
        let state = Process {
            pid,
            start_time: Instant::now(),
            is_shutdown: AtomicBool::new(false),
            wake_queue_sender,
            inner: Mutex::new(InnerProcess {
                rpc_clients: Slab::new(),
                rpc_servers: Slab::new(),
                io_objects: Slab::new(),
            }),
        };
        (state, wake_queue_receiver)
    }

    pub fn pid(&self) -> usize {
        self.pid
    }

    pub fn start_time(&self) -> Instant {
        self.start_time
    }

    pub fn is_shutdown(&self) -> bool {
        // TODO: Relax ordering?
        self.is_shutdown.load(Ordering::SeqCst)
    }

    pub fn shutdown(&self) {
        // TODO: Relax ordering?
        self.is_shutdown.store(true, Ordering::SeqCst);
    }

    pub fn wake_queue_sender(&self) -> &UnboundedSender<WakeParams> {
        &self.wake_queue_sender
    }

    pub unsafe fn io_read(
        &self,
        task_id: TaskId,
        io: u32,
        dst: *mut u8,
        len: u32,
    ) -> Result<Poll<u32>, Trap> {
        let mut inner = self.inner.lock().unwrap();
        let io = inner
            .io_objects
            .get_mut(io as _)
            .ok_or_else(|| Trap::new("bad IO handle"))?;
        Ok(unsafe { io.read(&self.wake_queue_sender, task_id, dst, len) })
    }

    pub unsafe fn io_write(
        &self,
        task_id: TaskId,
        io: u32,
        src: *const u8,
        len: u32,
    ) -> Result<Poll<u32>, Trap> {
        let mut inner = self.inner.lock().unwrap();
        let io = inner
            .io_objects
            .get_mut(io as _)
            .ok_or_else(|| Trap::new("bad IO handle"))?;
        Ok(unsafe { io.write(&self.wake_queue_sender, task_id, src, len) })
    }

    pub fn io_close(&self, io: u32) -> Result<(), Trap> {
        let io = self
            .inner
            .lock()
            .unwrap()
            .io_objects
            .try_remove(io as _)
            .ok_or_else(|| Trap::new("bad IO handle"))?;
        io.close();
        Ok(())
    }

    pub fn rpc_client_create(&self, service_name: String) -> u32 {
        self.inner
            .lock()
            .unwrap()
            .rpc_clients
            .insert(RpcClient::new(service_name))
            .try_into()
            .unwrap()
    }

    pub fn rpc_client_wait_healthy(
        arc_self: &Arc<Self>,
        task_id: TaskId,
        rpc_client: u32,
    ) -> Result<Poll<()>, Trap> {
        let inner = arc_self.inner.lock().unwrap();
        let rpc_client = inner
            .rpc_clients
            .get(rpc_client as _)
            .ok_or_else(|| Trap::new("bad RPC client handle"))?;

        Ok(SERVICE_REGISTRY.wait_for_server(
            arc_self,
            task_id,
            Cow::Borrowed(rpc_client.service_name()),
        ))
    }

    pub fn rpc_client_request(
        arc_self: &Arc<Self>,
        rpc_client: u32,
        method_name: &str,
    ) -> Result<(u32, u32), Trap> {
        let mut inner = arc_self.inner.lock().unwrap();
        let inner = &mut *inner;

        let rpc_client = inner
            .rpc_clients
            .get(rpc_client as _)
            .ok_or_else(|| Trap::new("bad RPC client handle"))?;

        let (request_reader, request_writer) = pipe();
        let (response_reader, response_writer) = pipe();
        let client_request_io = inner
            .io_objects
            .insert(IoObject::new_writer(request_writer))
            .try_into()
            .unwrap();
        let client_response_io = inner
            .io_objects
            .insert(IoObject::new_reader(response_reader))
            .try_into()
            .unwrap();

        // TODO: Introduce some kind of RPC channel abstraction and pick a healthy channel instead
        // of making this arbitrary pick all the way to the registry.

        let server_ref = SERVICE_REGISTRY.pick_server_or_die(rpc_client.service_name());
        let mut server_process_inner = server_ref.process.inner.lock().unwrap();
        let server_process_inner = &mut *server_process_inner;
        let server = &mut server_process_inner.rpc_servers[server_ref.rpc_server as _];
        let server_request_io = server_process_inner
            .io_objects
            .insert(IoObject::new_reader(request_reader))
            .try_into()
            .unwrap();
        let server_response_io = server_process_inner
            .io_objects
            .insert(IoObject::new_writer(response_writer))
            .try_into()
            .unwrap();
        server.queue_request(method_name, server_request_io, server_response_io);

        Ok((client_request_io, client_response_io))
    }

    pub fn rpc_server_create(arc_self: &Arc<Self>, params: &RpcServerParams) -> u32 {
        let mut inner = arc_self.inner.lock().unwrap();

        let method_names = params
            .methods
            .iter()
            .map(|method| method.method_name.clone())
            .collect();
        let id = inner
            .rpc_servers
            .insert(RpcServer::new(
                method_names,
                arc_self.wake_queue_sender.clone(),
            ))
            .try_into()
            .unwrap();
        SERVICE_REGISTRY.register(
            params.service_name.clone(),
            RpcServerRef {
                process: PointerIdentityArc::new(Arc::clone(arc_self)),
                rpc_server: id,
            },
        );
        id
    }

    pub fn rpc_server_get_request(
        &self,
        task_id: TaskId,
        rpc_server: u32,
    ) -> Result<Poll<RpcMetadata>, Trap> {
        let mut inner = self.inner.lock().unwrap();
        let rpc_server = inner
            .rpc_servers
            .get_mut(rpc_server as _)
            .ok_or_else(|| Trap::new("bad RPC server handle"))?;
        Ok(rpc_server.get_request(task_id))
    }
}
