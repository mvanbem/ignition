use ignition_9p::{FileType, Qid};
use lazy_static::lazy_static;

pub struct Node {
    qid: Qid,
}
impl Node {
    pub fn qid(&self) -> Qid {
        self.qid
    }
}

lazy_static! {
    pub static ref ROOT: Node = Node {
        qid: Qid {
            file_type: FileType::default().with_dir(true),
            path: 0,
            version: 0,
        },
    };
}
