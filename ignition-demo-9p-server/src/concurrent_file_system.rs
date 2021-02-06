#![allow(dead_code)]

use ignition_9p::OpenMode;
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};

/// A handle to a space in which [`Qid`]s are unique. One per 9p server.
#[derive(Clone)]
pub struct QidSpace {
    inner: Arc<Mutex<InnerQidSpace>>,
}
struct InnerQidSpace {
    next_path: u64,
}
impl QidSpace {
    pub fn new() -> QidSpace {
        QidSpace {
            inner: Arc::new(Mutex::new(InnerQidSpace { next_path: 0 })),
        }
    }

    fn allocate_path(&self) -> u64 {
        let mut inner = self.inner.lock().unwrap();
        let result = inner.next_path;
        inner.next_path += 1;
        result
    }
}

/// A handle to either a file or a directory.
pub trait Node {
    fn boxed_clone(&self) -> Box<dyn Node>;
    fn freeze(&self);
}

/// A handle to a directory.
#[derive(Clone)]
pub struct Directory {
    inner: Arc<Mutex<InnerDirectory>>,
}
pub struct InnerDirectory {
    qid_space: QidSpace,
    frozen: bool,
    /// None only during initialization.
    parent: Option<Directory>,
    name: String,
    qid_path: u64,
    qid_version: u32,
    entries: HashMap<String, Box<dyn Node>>,
}
impl Directory {
    pub fn new_root(qid_space: &QidSpace) -> Directory {
        let dir = Directory {
            inner: Arc::new(Mutex::new(InnerDirectory {
                qid_space: qid_space.clone(),
                frozen: false,
                parent: None,
                name: "/".to_string(),
                qid_path: qid_space.allocate_path(),
                qid_version: 0,
                entries: HashMap::new(),
            })),
        };
        {
            let mut inner = dir.inner.lock().unwrap();
            inner.parent = Some(Directory {
                inner: Arc::clone(&dir.inner),
            });
        }
        dir
    }
    pub fn create(&self, name: &str, perm: u32, _mode: OpenMode) -> Result<Box<dyn Node>, FsError> {
        let inner = self.inner.lock().unwrap();
        if inner.frozen {
            return fs_error("frozen");
        }
        if perm & 0x80000000 == 0x80000000 {
            // Creating a directory.
            if inner.entries.contains_key(name) {
                return fs_error("already exists");
            }
        } else {
            // Creating a file.
        }
        fs_error("idk")
    }
}
impl Node for Directory {
    fn boxed_clone(&self) -> Box<dyn Node> {
        Box::new(self.clone())
    }
    fn freeze(&self) {
        let mut inner = self.inner.lock().unwrap();
        if !inner.frozen {
            inner.frozen = true;
            // TODO: recurse into children when those exist
        }
    }
}

#[derive(Debug)]
pub struct FsError(&'static str);
impl std::fmt::Display for FsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Error for FsError {}

fn fs_error<T>(msg: &'static str) -> Result<T, FsError> {
    Err(FsError(msg))
}
