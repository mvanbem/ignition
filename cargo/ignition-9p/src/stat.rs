use crate::ext::{ReadBytesExt, WriteBytesExt};
use crate::{DontTouch, Qid, StatMode};
use std::{convert::TryInto, io};

/// A machine-independent directory entry.
///
/// See also: [stat(5) in the Plan 9 Manual](http://man.cat-v.org/plan_9/5/stat)
#[derive(Clone, Debug, Eq, PartialEq)]
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
impl Stat {
    pub fn read<R: io::Read>(r: &mut R) -> io::Result<Stat> {
        let data = r.read_u16_prefixed_bytes()?;
        let mut r: &[u8] = &data;

        let embedded_size = r.read_u16()?;
        if embedded_size as usize != r.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "stat size mismatch",
            ));
        }

        let kernel_type = r.read_u16()?;
        let kernel_dev = r.read_u32()?;
        let qid = Qid::read(&mut r)?;
        let mode = StatMode(r.read_u32()?);
        let atime = r.read_u32()?;
        let mtime = r.read_u32()?;
        let length = r.read_u64()?;
        let name = r.read_string()?;
        let uid = r.read_string()?;
        let gid = r.read_string()?;
        let muid = r.read_string()?;

        if r.len() != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unexpected data after stat: {} bytes", r.len()),
            ));
        }

        Ok(Stat {
            kernel_type,
            kernel_dev,
            qid,
            mode,
            atime,
            mtime,
            length,
            name,
            uid,
            gid,
            muid,
        })
    }

    pub fn write<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        let size = self
            .size()
            .try_into()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        w.write_u16(size + 2)?;
        w.write_u16(size)?;
        w.write_u16(self.kernel_type)?;
        w.write_u32(self.kernel_dev)?;
        self.qid.write(w)?;
        w.write_u32(self.mode.0)?;
        w.write_u32(self.atime)?;
        w.write_u32(self.mtime)?;
        w.write_u64(self.length)?;
        w.write_string(&self.name)?;
        w.write_string(&self.uid)?;
        w.write_string(&self.gid)?;
        w.write_string(&self.muid)?;
        Ok(())
    }

    /// Returns the serialized size of the entry, including all fields and the u16 length prefix.
    fn size(&self) -> usize {
        47 + self.name.len() + self.uid.len() + self.gid.len() + self.muid.len()
    }
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

#[cfg(test)]
mod tests {
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
        expected.write(&mut data).unwrap();
        let mut r: &[u8] = &data;
        assert_eq!(Stat::read(&mut r).unwrap(), expected);
        assert_eq!(r.len(), 0);
    }
}
