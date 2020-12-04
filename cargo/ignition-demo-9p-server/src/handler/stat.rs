use std::error::Error;
use std::sync::{Arc, Mutex};

use ignition_9p::message::{MessageBody, RStat, TStat};

use super::rerror;
use crate::connection_state::ConnectionState;

pub fn handle_stat(
    state: &Arc<Mutex<ConnectionState>>,
    req: &TStat,
) -> Result<MessageBody, Box<dyn Error>> {
    let state = state.lock().unwrap();
    let node = match state.get_fid(req.fid) {
        Some(node) => node,
        None => return rerror("fid not in use"),
    };
    Ok(MessageBody::RStat(RStat { stat: node.stat() }))
}
