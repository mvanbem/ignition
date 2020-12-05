use ignition_9p::message::{MessageBody, RVersion, TVersion};
use ignition_9p::Tag;
use std::error::Error;
use std::sync::{Arc, Mutex};

use super::rerror;
use crate::connection_state::ConnectionState;

const SUPPORTED_VERSION: &'static str = "9P2000";

pub fn handle_version(
    state: &Arc<Mutex<ConnectionState>>,
    tag: Tag,
    req: &TVersion,
) -> Result<MessageBody, Box<dyn Error>> {
    if tag != Tag::NOTAG {
        return rerror("expected NOTAG in Tversion request");
    }
    let msize = std::cmp::min(req.msize, crate::MAX_MSIZE);
    if req.version != SUPPORTED_VERSION {
        // 9p2000 is the only supported protocol version.
        log::warn!(
            "peer requested unsupported protocol version {:?}",
            req.version
        );
        return Ok(MessageBody::RVersion(RVersion {
            msize,
            version: "unknown".to_string(),
        }));
    }

    state.lock().unwrap().reset();

    Ok(MessageBody::RVersion(RVersion {
        msize,
        version: SUPPORTED_VERSION.to_string(),
    }))
}
