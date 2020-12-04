use ignition_9p::message::{Message, MessageBody, RError};
use std::error::Error;
use std::sync::{Arc, Mutex};

use crate::connection_state::ConnectionState;

mod attach;
mod clunk;
mod stat;
mod version;

use attach::handle_attach;
use clunk::handle_clunk;
use stat::handle_stat;
use version::handle_version;

fn rerror<T: Into<String>>(msg: T) -> Result<MessageBody, Box<dyn Error>> {
    Ok(MessageBody::RError(RError { ename: msg.into() }))
}

pub fn handle_request(
    state: &Arc<Mutex<ConnectionState>>,
    req: &Message,
) -> Result<MessageBody, Box<dyn Error>> {
    match req.body {
        MessageBody::TVersion(ref tversion) => handle_version(state, req.tag, tversion),
        MessageBody::TAttach(ref tattach) => handle_attach(state, tattach),
        MessageBody::TStat(ref tstat) => handle_stat(state, tstat),
        MessageBody::TClunk(ref tclunk) => handle_clunk(state, tclunk),
        _ => {
            log::warn!("unsupported message type");
            rerror("unsupported message type")
        }
    }
}
