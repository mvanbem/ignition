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

pub struct ProcessState {
    pid: usize,
    is_shutdown: AtomicBool,
    wake_queue_sender: UnboundedSender<WakeParams>,
    start_time: Instant,
    rpc_clients: Slab<RpcClient>,
    rpc_servers: Slab<RpcServer>,
    io_objects: Slab<IoObject>,
}

impl ProcessState {
    pub fn new(pid: usize) -> (Self, UnboundedReceiver<WakeParams>) {
        let (wake_queue_sender, wake_queue_receiver) = unbounded_channel();
        let state = ProcessState {
            pid,
            is_shutdown: AtomicBool::new(false),
            wake_queue_sender,
            start_time: Instant::now(),
            rpc_clients: Slab::new(),
            rpc_servers: Slab::new(),
            io_objects: Slab::new(),
        };
        (state, wake_queue_receiver)
    }

    pub fn pid(&self) -> usize {
        self.pid
    }

    pub fn shutdown(&self) {
        // TODO: Relax ordering?
        self.is_shutdown.store(true, Ordering::SeqCst);
    }

    pub fn is_shutdown(&self) -> bool {
        // TODO: Relax ordering?
        self.is_shutdown.load(Ordering::SeqCst)
    }

    pub fn wake_queue_sender(&self) -> &UnboundedSender<WakeParams> {
        &self.wake_queue_sender
    }

    pub fn start_time(&self) -> Instant {
        self.start_time
    }

    pub unsafe fn io_read(
        &mut self,
        task_id: TaskId,
        io: u32,
        dst: *mut u8,
        len: u32,
    ) -> Result<Poll<u32>, Trap> {
        let io = self
            .io_objects
            .get_mut(io as _)
            .ok_or_else(|| Trap::new("bad IO handle"))?;
        Ok(unsafe { io.read(&self.wake_queue_sender, task_id, dst, len) })
    }

    pub unsafe fn io_write(
        &mut self,
        task_id: TaskId,
        io: u32,
        src: *const u8,
        len: u32,
    ) -> Result<Poll<u32>, Trap> {
        let io = self
            .io_objects
            .get_mut(io as _)
            .ok_or_else(|| Trap::new("bad IO handle"))?;
        Ok(unsafe { io.write(&self.wake_queue_sender, task_id, src, len) })
    }

    pub fn io_close(&mut self, io: u32) -> Result<(), Trap> {
        let io = self
            .io_objects
            .try_remove(io as _)
            .ok_or_else(|| Trap::new("bad IO handle"))?;
        io.close();
        Ok(())
    }

    pub fn rpc_client_create(&mut self, service_name: String) -> u32 {
        self.rpc_clients
            .insert(RpcClient::new(service_name))
            .try_into()
            .unwrap()
    }

    pub fn rpc_client_wait_healthy(
        arc_this: &Arc<Mutex<Self>>,
        task_id: TaskId,
        rpc_client: u32,
    ) -> Result<Poll<()>, Trap> {
        let mut this_guard = arc_this.lock().unwrap();
        let this = &mut *this_guard;

        let rpc_client = this
            .rpc_clients
            .get(rpc_client as _)
            .ok_or_else(|| Trap::new("bad RPC client handle"))?;

        Ok(SERVICE_REGISTRY.wait_for_server(
            arc_this,
            task_id,
            Cow::Borrowed(rpc_client.service_name()),
        ))
    }

    pub fn rpc_client_request(
        arc_this: &Arc<Mutex<Self>>,
        rpc_client: u32,
        method_name: &str,
    ) -> Result<(u32, u32), Trap> {
        let mut this_guard = arc_this.lock().unwrap();
        let this = &mut *this_guard;

        let rpc_client = this
            .rpc_clients
            .get(rpc_client as _)
            .ok_or_else(|| Trap::new("bad RPC client handle"))?;

        let (request_reader, request_writer) = pipe();
        let (response_reader, response_writer) = pipe();
        let client_request_io = this
            .io_objects
            .insert(IoObject::new_writer(request_writer))
            .try_into()
            .unwrap();
        let client_response_io = this
            .io_objects
            .insert(IoObject::new_reader(response_reader))
            .try_into()
            .unwrap();

        // TODO: Introduce some kind of RPC channel abstraction and pick a healthy channel instead
        // of making this arbitrary pick all the way to the registry.

        let server_ref = SERVICE_REGISTRY.pick_server_or_die(rpc_client.service_name());
        let mut server_process_guard = server_ref.process.lock().unwrap();
        let server_process = &mut *server_process_guard;
        let server = &mut server_process.rpc_servers[server_ref.rpc_server as _];
        let server_request_io = server_process
            .io_objects
            .insert(IoObject::new_reader(request_reader))
            .try_into()
            .unwrap();
        let server_response_io = server_process
            .io_objects
            .insert(IoObject::new_writer(response_writer))
            .try_into()
            .unwrap();
        server.queue_request(method_name, server_request_io, server_response_io);

        Ok((client_request_io, client_response_io))
    }

    pub fn rpc_server_create(arc_this: &Arc<Mutex<Self>>, params: &RpcServerParams) -> u32 {
        let mut this_guard = arc_this.lock().unwrap();
        let this = &mut *this_guard;

        let method_names = params
            .methods
            .iter()
            .map(|method| method.method_name.clone())
            .collect();
        let id = this
            .rpc_servers
            .insert(RpcServer::new(method_names, this.wake_queue_sender.clone()))
            .try_into()
            .unwrap();
        SERVICE_REGISTRY.register(
            params.service_name.clone(),
            RpcServerRef {
                process: PointerIdentityArc::new(Arc::clone(arc_this)),
                rpc_server: id,
            },
        );
        id
    }

    pub fn rpc_server_get_request(
        &mut self,
        task_id: TaskId,
        rpc_server: u32,
    ) -> Result<Poll<RpcMetadata>, Trap> {
        let rpc_server = self
            .rpc_servers
            .get_mut(rpc_server as _)
            .ok_or_else(|| Trap::new("bad RPC server handle"))?;
        Ok(rpc_server.get_request(task_id))
    }
}
