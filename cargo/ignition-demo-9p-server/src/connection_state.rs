use std::{collections::HashMap, error::Error, fmt::Display};

use ignition_9p::Fid;

use crate::static_file_system::Node;

pub struct ConnectionState {
    fids: HashMap<u32, &'static Node>,
}
impl ConnectionState {
    pub fn new() -> ConnectionState {
        ConnectionState {
            fids: HashMap::new(),
        }
    }

    pub fn reset(&mut self) {
        self.fids.clear();
    }
    pub fn allocate_fid(&mut self, fid: Fid, node: &'static Node) -> Result<(), AllocateFidError> {
        if !self.fids.contains_key(&fid.0) {
            self.fids.insert(fid.0, node);
            Ok(())
        } else {
            Err(AllocateFidError::FidAlreadyInUse { fid })
        }
    }
    pub fn get_fid(&self, fid: Fid) -> Option<&'static Node> {
        self.fids.get(&fid.0).map(|x| *x)
    }
    pub fn remove_fid(&mut self, fid: Fid) -> Option<&'static Node> {
        self.fids.remove(&fid.0)
    }
}

#[derive(Debug)]
pub enum AllocateFidError {
    FidAlreadyInUse { fid: Fid },
}
impl Display for AllocateFidError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AllocateFidError::FidAlreadyInUse { fid } => write!(f, "fid {} already in use", fid.0),
        }
    }
}
impl Error for AllocateFidError {}
