use std::error::Error;
use std::sync::{Arc, Mutex};

use ignition_9p::message::{MessageBody, RAttach, TAttach};
use ignition_9p::{FileType, Qid};

use super::rerror;
use crate::connection_state::{AllocateFidError, ConnectionState};
use crate::static_file_system;

pub fn handle_attach(
    state: &Arc<Mutex<ConnectionState>>,
    req: &TAttach,
) -> Result<MessageBody, Box<dyn Error>> {
    if req.aname != "" {
        return rerror("not found");
    }
    match state
        .lock()
        .unwrap()
        .allocate_fid(req.fid, static_file_system::ROOT.qid())
    {
        Ok(()) => (),
        Err(e @ AllocateFidError::FidAlreadyInUse { .. }) => return rerror(e.to_string()),
    }
    Ok(MessageBody::RAttach(RAttach {
        qid: Qid {
            file_type: FileType::default().with_dir(true),
            version: 0, // TODO: resolve hack
            path: 0,    // TODO: resolve hack
        },
    }))
}
