use crate::ext::{ReadBytesExt, WriteBytesExt};
use crate::{DontTouch, FileType};
use std::io;

/// Represents a server's unique identification for a file.
///
/// Two files on the same server hierarchy are the same if and only if their qids are the same. (The
/// client may have multiple fids pointing to a single file on a server and hence having a single
/// qid.)
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Qid {
    /// Specifies whether the file is a directory, append-only file, etc.
    pub file_type: FileType,
    /// Version number for a file.
    ///
    /// Typically, it is incremented every time the file is modified.
    pub version: u32,
    /// An integer unique among all files in the hierarchy.
    ///
    /// If a file is deleted and recreated with the same name in the same directory, the old and new
    /// path components of the qids should be different.
    pub path: u64,
}
impl Qid {
    pub fn read<R: io::Read>(r: &mut R) -> io::Result<Qid> {
        let file_type = FileType(r.read_u8()?);
        let version = r.read_u32()?;
        let path = r.read_u64()?;
        Ok(Qid {
            file_type,
            version,
            path,
        })
    }
    pub fn write<W: io::Write>(self, w: &mut W) -> io::Result<()> {
        w.write_u8(self.file_type.0)?;
        w.write_u32(self.version)?;
        w.write_u64(self.path)
    }
}
impl DontTouch for Qid {
    fn dont_touch() -> Self {
        Qid {
            file_type: FileType(DontTouch::dont_touch()),
            version: DontTouch::dont_touch(),
            path: DontTouch::dont_touch(),
        }
    }
}
