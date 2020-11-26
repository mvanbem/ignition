use std::io;

pub mod ext;
pub mod message;
pub mod mode;

use crate::ext::{ReadBytesExt, WriteBytesExt};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Fid(pub u32);
impl Fid {
    pub const NOFID: Fid = Fid(!0);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Qid {
    pub file_type: u8,
    pub version: u32,
    pub path: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stat {
    pub kernel_type: u16,
    pub kernel_dev: u32,
    pub qid: Qid,
    pub mode: u32,
    pub atime: u32,
    pub mtime: u32,
    pub length: u64,
    pub name: String,
    pub uid: String,
    pub gid: String,
    pub muid: String,
}
impl Stat {
    pub const KERNEL_TYPE_DONT_TOUCH: u16 = !0;
    pub const KERNEL_DEV_DONT_TOUCH: u32 = !0;
    pub const QID_DONT_TOUCH: Qid = Qid {
        file_type: !0,
        version: !0,
        path: !0,
    };
    pub const MODE_DONT_TOUCH: u32 = !0;
    pub const ATIME_DONT_TOUCH: u32 = !0;
    pub const MTIME_DONT_TOUCH: u32 = !0;
    pub const LENGTH_DONT_TOUCH: u64 = !0;

    pub const DMDIR: u32 = 0x80000000;
    pub const DMAPPEND: u32 = 0x40000000;
    pub const DMEXCL: u32 = 0x20000000;
    pub const DMTMP: u32 = 0x04000000;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Tag(pub u16);
impl Tag {
    pub fn read<R: io::Read>(r: &mut R) -> io::Result<Tag> {
        Ok(Tag(r.read_u16()?))
    }

    pub fn write<W: io::Write>(self, w: &mut W) -> io::Result<()> {
        w.write_u16(self.0)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
