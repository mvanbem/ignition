use std::error::Error;
use std::sync::{Arc, Mutex};

use ignition_9p::message::{MessageBody, TClunk};

use super::rerror;
use crate::connection_state::ConnectionState;

pub fn handle_clunk(
    state: &Arc<Mutex<ConnectionState>>,
    req: &TClunk,
) -> Result<MessageBody, Box<dyn Error>> {
    let mut state = state.lock().unwrap();
    match state.remove_fid(req.fid) {
        Some(_) => (),
        None => return rerror("fid not in use"),
    }
    Ok(MessageBody::RClunk)
}
