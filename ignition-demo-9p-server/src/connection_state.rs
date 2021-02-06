use ignition_9p::{message::*, Fid, FileType, OpenAccess, Qid, Tag};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::sync::{Arc, Mutex};
use thiserror::Error;

use crate::file_system::{FileSystem, Node};

const SUPPORTED_VERSION: &'static str = "9P2000";

fn rerror<T: Into<String>, E>(msg: T) -> Result<MessageBody, E> {
    Ok(MessageBody::RError(RError { ename: msg.into() }))
}

pub struct ConnectionState {
    inner: Arc<Mutex<InnerConnectionState>>,
}

struct InnerConnectionState {
    fs: &'static FileSystem,
    fids: HashMap<u32, FidState>,
}

impl ConnectionState {
    pub fn new(fs: &'static FileSystem) -> ConnectionState {
        ConnectionState {
            inner: Arc::new(Mutex::new(InnerConnectionState {
                fs,
                fids: HashMap::new(),
            })),
        }
    }

    pub async fn handle_request(&self, req: &Message) -> Result<MessageBody, HandleRequestError> {
        match req.body {
            MessageBody::TVersion(TVersion { msize, ref version }) => {
                if req.tag != Tag::NOTAG {
                    return rerror("expected NOTAG in Tversion request");
                }
                let msize = std::cmp::min(msize, crate::MAX_MSIZE);
                if version != SUPPORTED_VERSION {
                    // 9p2000 is the only supported protocol version.
                    log::warn!("peer requested unsupported protocol version {:?}", version);
                    return Ok(MessageBody::RVersion(RVersion {
                        msize,
                        version: "unknown".to_string(),
                    }));
                }

                self.inner.lock().unwrap().reset();

                Ok(MessageBody::RVersion(RVersion {
                    msize,
                    version: SUPPORTED_VERSION.to_string(),
                }))
            }

            MessageBody::TAttach(TAttach {
                fid,
                afid: _,
                uname: _,
                ref aname,
            }) => {
                if aname != "" {
                    return rerror("not found");
                }
                let mut inner = self.inner.lock().unwrap();
                let fs: &'static FileSystem = inner.fs;
                match inner.allocate_fid(fid, Node::Directory(fs.root())) {
                    Ok(()) => (),
                    Err(e @ AllocateFidError::FidAlreadyInUse { .. }) => {
                        return rerror(e.to_string())
                    }
                }
                Ok(MessageBody::RAttach(RAttach {
                    qid: Qid {
                        file_type: FileType::default().with_dir(true),
                        version: 0, // TODO: resolve hack
                        path: 0,    // TODO: resolve hack
                    },
                }))
            }

            MessageBody::TWalk(TWalk {
                fid,
                newfid,
                ref names,
            }) => {
                let mut state = self.inner.lock().unwrap();
                let mut node = match state.fids.get(&fid.0) {
                    Some(fid_state) => fid_state.node.clone(),
                    None => return rerror("fid not in use"),
                };
                if newfid != fid {
                    if let Some(_) = state.fids.get(&newfid.0) {
                        return rerror("fid already in use");
                    }
                }

                let mut qids = vec![];
                for (i, name) in names.iter().enumerate() {
                    let directory = match node {
                        Node::Directory(directory) => directory,
                        _ => {
                            // We are attempting to walk from a file, which is forbidden. Per the
                            // protocol, bail with an error on the first iteration, but just stop on
                            // later iterations.
                            if i == 0 {
                                return rerror("cannot walk with non-empty path from a file");
                            } else {
                                break;
                            }
                        }
                    };
                    node = if name == ".." {
                        Node::Directory(directory.parent())
                    } else {
                        match directory.entry(name) {
                            Some(node) => node,
                            // TODO: support permission denied
                            None => {
                                // Not found. Per the protocol, bail with an error on the first
                                // iteration, but just stop on later iterations.
                                if i == 0 {
                                    return rerror("not found");
                                } else {
                                    break;
                                }
                            }
                        }
                    };
                    qids.push(node.qid());
                }

                state.set_fid(newfid, node);
                Ok(MessageBody::RWalk(RWalk { qids }))
            }

            MessageBody::TOpen(TOpen { fid, mode }) => {
                let mut state = self.inner.lock().unwrap();
                let fid_state = match state.fids.get_mut(&fid.0) {
                    Some(fid_state) => fid_state,
                    None => return rerror("fid not in use"),
                };

                // TODO: not super fake
                if mode.access() == OpenAccess::WRITE
                    || mode.access() == OpenAccess::RDWR
                    || mode.trunc()
                    || mode.rclose()
                {
                    return rerror("permission denied");
                }

                fid_state.is_open = true;
                Ok(MessageBody::ROpen(ROpen {
                    qid: fid_state.node.qid(),
                    iounit: 0,
                }))
            }

            // MessageBody::TCreate(_) => {}
            //
            MessageBody::TRead(TRead { fid, offset, count }) => {
                let state = self.inner.lock().unwrap();
                let fid_state = match state.fids.get(&fid.0) {
                    Some(fid_state) => fid_state,
                    None => return rerror("fid not in use"),
                };
                if !fid_state.is_open {
                    return rerror("fid not open");
                };

                // Early out if reading from an offset past the end of the content.
                let content = fid_state.node.content();
                let offset = usize::try_from(offset).unwrap();
                let count = usize::try_from(count).unwrap();
                if offset >= content.len() {
                    return Ok(MessageBody::RRead(RRead { data: vec![] }));
                }

                // Adjust count to prevent reads past the end of the content. The number of bytes
                // available will always be at least one because of the early out above.
                let available = content.len() - offset;
                let count = std::cmp::min(count, available);

                // For directories, adjust count to prevent reading less than a whole stat record.
                // This is required by protocol.
                let count = if let Some(cut_points) = fid_state.node.cut_points() {
                    match cut_points.binary_search(&(offset + count)) {
                        Ok(_) => {
                            // Exact match. This read was already aligned to end on a cut point. No
                            // change required.
                            count
                        }
                        Err(next_cut_point_index) => {
                            // No exact match. The given index refers to the next cut point after
                            // the end of the current proposed read. Limit this read to the cut
                            // point *one before* that one.
                            //
                            //
                            //        +--- offset
                            //        |            +--- offset + count
                            //        |            |
                            // [------o========|===o---|--- ~ ]
                            //                 |       |
                            //                 |       +--- cut_points[next_cut_point_index]
                            //                 +--- cut_points[next_cut_point_index - 1]
                            //
                            // NOTE: Subtracting one will never underflow because the argument to
                            // binary_search will never be zero (count is nonzero) and the
                            // cut_points vec's first element is zero, so the given index will be at
                            // least one.
                            let new_end = cut_points[next_cut_point_index - 1];
                            match new_end.checked_sub(offset) {
                                Some(adjusted_count) => adjusted_count,
                                None => {
                                    return Err(HandleRequestError::ProtocolError(format!(
                                        "directory read originated between stat entries"
                                    )));
                                }
                            }
                        }
                    }
                } else {
                    // This is a file read so there is no need to enforce this directory content
                    // nonsense.
                    count
                };

                // Perform the read.
                let mut data = vec![];
                data.extend_from_slice(&fid_state.node.content()[offset..offset + count]);
                Ok(MessageBody::RRead(RRead { data }))
            }

            // MessageBody::TWrite(_) => {}
            //
            MessageBody::TClunk(TClunk { fid }) => {
                let mut state = self.inner.lock().unwrap();
                match state.fids.remove(&fid.0) {
                    Some(_) => (),
                    None => return rerror("fid not in use"),
                }
                Ok(MessageBody::RClunk)
            }
            MessageBody::TStat(TStat { fid }) => {
                let state = self.inner.lock().unwrap();
                let fid_state = match state.fids.get(&fid.0) {
                    Some(fid_state) => fid_state,
                    None => return rerror("fid not in use"),
                };
                Ok(MessageBody::RStat(RStat {
                    stat: fid_state.node.stat(),
                }))
            }

            // MessageBody::TWstat(_) => {}
            //
            MessageBody::RVersion(_)
            | MessageBody::RAttach(_)
            | MessageBody::RError(_)
            | MessageBody::RWalk(_)
            | MessageBody::ROpen(_)
            | MessageBody::RCreate(_)
            | MessageBody::RRead(_)
            | MessageBody::RWrite(_)
            | MessageBody::RClunk
            | MessageBody::RStat(_)
            | MessageBody::RWstat => Err(HandleRequestError::ProtocolError(format!(
                "received a reply message (type {:?})",
                req.body.message_type()
            ))),

            _ => Err(HandleRequestError::NotImplemented(format!(
                "message type {:?}",
                req.body.message_type()
            ))),
        }
    }
}

impl InnerConnectionState {
    pub fn reset(&mut self) {
        self.fids.clear();
    }

    pub fn allocate_fid(&mut self, fid: Fid, node: Node<'static>) -> Result<(), AllocateFidError> {
        if !self.fids.contains_key(&fid.0) {
            self.fids.insert(fid.0, FidState::new(node));
            Ok(())
        } else {
            Err(AllocateFidError::FidAlreadyInUse { fid })
        }
    }

    pub fn set_fid(&mut self, fid: Fid, node: Node<'static>) {
        self.fids.insert(fid.0, FidState::new(node));
    }
}

#[derive(Error, Debug)]
pub enum HandleRequestError {
    #[error("peer broke protocol: {0}")]
    ProtocolError(String),

    #[error("not implemented: {0}")]
    NotImplemented(String),
}

struct FidState {
    node: Node<'static>,
    is_open: bool,
}

impl FidState {
    fn new(node: Node<'static>) -> FidState {
        FidState {
            node,
            is_open: false,
        }
    }
}

#[derive(Debug, Error)]
pub enum AllocateFidError {
    #[error("fid {} already in use", .fid.0)]
    FidAlreadyInUse { fid: Fid },
}
