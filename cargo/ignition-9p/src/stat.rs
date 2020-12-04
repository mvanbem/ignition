use crate::wire::{EmbeddedSize, SerializedSize};
use crate::{DontTouch, Qid, StatMode};
use ignition_9p_wire_derive::{ReadWireFormat, WriteWireFormat};

/// A machine-independent directory entry.
///
/// See also: [stat(5) in the Plan 9 Manual](http://man.cat-v.org/plan_9/5/stat)
#[derive(Clone, Debug, Eq, PartialEq, ReadWireFormat, WriteWireFormat)]
#[ignition_9p_wire(embedded_size_prefix = "u16")]
pub struct Stat {
    /// For kernel use.
    pub kernel_type: u16,
    /// For kernel use.
    pub kernel_dev: u32,
    /// The type of the file (directory, etc.), represented as a bit vector corresponding to the
    /// high 8 bits of the file's mode word.
    pub qid: Qid,
    /// Permissions and flags.
    pub mode: StatMode,
    /// Last access time.
    ///
    /// This field records the last read of the contents. It is set whenever mtime is set. In
    /// addition, for a directory, it is set by an attach, walk, or create, all whether successful
    /// or not.
    pub atime: u32,
    /// Last modification time.
    ///
    /// The mtime field reflects the time of the last change of content (except when later changed
    /// by wstat).  For a plain file, mtime is the time of the most recent create, open with
    /// truncation, or write; for a directory it is the time of the most recent remove, create, or
    /// wstat of a file in the directory.
    pub mtime: u32,
    /// Length of file in bytes.
    ///
    /// Directories and most files representing devices have a conventional length of 0.
    pub length: u64,
    /// File name. Must be `"/"` if the file is the root directory of the server.
    pub name: String,
    /// Owner name.
    pub uid: String,
    /// Group name.
    pub gid: String,
    /// Name of the user who last modified (changed the mtime of) the file.
    pub muid: String,
}
impl DontTouch for Stat {
    fn dont_touch() -> Stat {
        Stat {
            kernel_type: DontTouch::dont_touch(),
            kernel_dev: DontTouch::dont_touch(),
            qid: DontTouch::dont_touch(),
            mode: DontTouch::dont_touch(),
            atime: DontTouch::dont_touch(),
            mtime: DontTouch::dont_touch(),
            length: DontTouch::dont_touch(),
            name: DontTouch::dont_touch(),
            uid: DontTouch::dont_touch(),
            gid: DontTouch::dont_touch(),
            muid: DontTouch::dont_touch(),
        }
    }
}
impl EmbeddedSize for Stat {
    fn embedded_size(&self) -> usize {
        self.serialized_size() - 2
    }
}
impl SerializedSize for Stat {
    fn serialized_size(&self) -> usize {
        49 + self.name.len() + self.uid.len() + self.gid.len() + self.muid.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::wire::{ReadWireFormat, WriteWireFormat};
    use crate::{FileType, Qid, Stat, StatMode, UnixTriplet};

    #[test]
    fn roundtrip() {
        let file_type = FileType::default().with_dir(true);
        let expected = Stat {
            kernel_type: 0,
            kernel_dev: 0,
            qid: Qid {
                file_type,
                version: 0,
                path: 0,
            },
            mode: StatMode::default()
                .with_file_type(file_type)
                .with_user(UnixTriplet::RWX)
                .with_group(UnixTriplet::RWX)
                .with_other(UnixTriplet::RX),
            atime: 0,
            mtime: 0,
            length: 0,
            name: "/".to_string(),
            uid: "root".to_string(),
            gid: "root".to_string(),
            muid: "root".to_string(),
        };

        let mut data = vec![];
        expected.write_to(&mut data).unwrap();
        let mut r: &[u8] = &data;
        assert_eq!(Stat::read_from(&mut r).unwrap(), expected);
        assert_eq!(r.len(), 0);
    }
}
