use std::{collections::HashMap, error::Error, fmt::Display};

pub struct ConnectionState {
    fids: HashMap<u32, ignition_9p::Qid>,
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
    pub fn allocate_fid(
        &mut self,
        fid: ignition_9p::Fid,
        qid: ignition_9p::Qid,
    ) -> Result<(), AllocateFidError> {
        if !self.fids.contains_key(&fid.0) {
            self.fids.insert(fid.0, qid);
            Ok(())
        } else {
            Err(AllocateFidError::FidAlreadyInUse { fid })
        }
    }
}

#[derive(Debug)]
pub enum AllocateFidError {
    FidAlreadyInUse { fid: ignition_9p::Fid },
}
impl Display for AllocateFidError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AllocateFidError::FidAlreadyInUse { fid } => write!(f, "fid {} already in use", fid.0),
        }
    }
}
impl Error for AllocateFidError {}
