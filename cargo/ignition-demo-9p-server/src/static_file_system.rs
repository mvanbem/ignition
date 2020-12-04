use ignition_9p::{FileType, Qid, Stat, StatMode, UnixTriplet};
use lazy_static::lazy_static;

pub struct Node {
    qid: Qid,
    name: String,
}
impl Node {
    pub fn stat(&self) -> Stat {
        Stat {
            kernel_type: 0,
            kernel_dev: 0,
            qid: self.qid,
            mode: if self.qid.file_type.dir() {
                StatMode::default()
                    .with_file_type(self.qid.file_type)
                    .with_user(UnixTriplet::RWX)
                    .with_group(UnixTriplet::RX)
                    .with_other(UnixTriplet::RX)
            } else {
                StatMode::default()
                    .with_file_type(self.qid.file_type)
                    .with_user(UnixTriplet::RW)
                    .with_group(UnixTriplet::R)
                    .with_other(UnixTriplet::R)
            },
            atime: 0,
            mtime: 0,
            length: 0,
            name: self.name.clone(),
            uid: "root".to_string(),
            gid: "root".to_string(),
            muid: "root".to_string(),
        }
    }
}

lazy_static! {
    pub static ref ROOT: Node = Node {
        qid: Qid {
            file_type: FileType::default().with_dir(true),
            path: 0,
            version: 0,
        },
        name: "/".to_string(),
    };
}
