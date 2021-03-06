use crate::wire::{ReadFrom, WriteTo};
use derive_more::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign};
use std::io::{self, Read, Write};

bitfield::bitfield! {
    /// A file type as used in [`Qid::file_type`](crate::Qid::file_type) or the upper bits of
    /// [`StatMode`](crate::StatMode).
    #[derive(
        BitAnd,
        BitAndAssign,
        BitOr,
        BitOrAssign,
        BitXor,
        BitXorAssign,
        Clone,
        Copy,
        Default,
        Eq,
        PartialEq,
        // TODO: Derive ReadWireFormat, WriteWireFormat. At the moment they seem to conflict with
        // `bitfield!` and I'm not sure why.
    )]
    pub struct FileType(u8);
    pub tmp, set_tmp: 2;
    pub auth, set_auth: 3;
    pub excl, set_excl: 5;
    pub append, set_append: 6;
    pub dir, set_dir: 7;
}
impl FileType {
    pub const FILE: FileType = FileType(0x00);
    pub const TMP: FileType = FileType(0x04);
    pub const AUTH: FileType = FileType(0x08);
    pub const EXCL: FileType = FileType(0x20);
    pub const APPEND: FileType = FileType(0x40);
    pub const DIR: FileType = FileType(0x80);

    pub fn with_tmp(mut self, tmp: bool) -> Self {
        self.set_tmp(tmp);
        self
    }
    pub fn with_auth(mut self, auth: bool) -> Self {
        self.set_auth(auth);
        self
    }
    pub fn with_excl(mut self, excl: bool) -> Self {
        self.set_excl(excl);
        self
    }
    pub fn with_append(mut self, append: bool) -> Self {
        self.set_append(append);
        self
    }
    pub fn with_dir(mut self, dir: bool) -> Self {
        self.set_dir(dir);
        self
    }
}
impl std::fmt::Debug for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FileType(")?;
        if self.dir() {
            write!(f, "DIR")?;
        } else {
            write!(f, "FILE")?;
        }
        if self.tmp() {
            write!(f, "|TMP")?;
        }
        if self.auth() {
            write!(f, "|AUTH")?;
        }
        if self.excl() {
            write!(f, "|EXCL")?;
        }
        if self.append() {
            write!(f, "|APPEND")?;
        }
        write!(f, ")")
    }
}
impl From<FileType> for u8 {
    fn from(mode: FileType) -> Self {
        mode.0
    }
}
impl From<FileType> for u32 {
    fn from(mode: FileType) -> Self {
        mode.0 as u32
    }
}
impl From<u8> for FileType {
    fn from(value: u8) -> Self {
        FileType(value)
    }
}
impl From<u32> for FileType {
    fn from(value: u32) -> Self {
        FileType(value as u8)
    }
}
impl ReadFrom for FileType {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
        Ok(FileType(ReadFrom::read_from(r)?))
    }
}
impl WriteTo for FileType {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        self.0.write_to(w)
    }
}

#[cfg(test)]
mod tests {
    use super::FileType;

    #[test]
    fn constants() {
        assert_eq!(FileType::TMP.tmp(), true);
        assert_eq!(FileType::AUTH.auth(), true);
        assert_eq!(FileType::EXCL.excl(), true);
        assert_eq!(FileType::APPEND.append(), true);
        assert_eq!(FileType::DIR.dir(), true);
    }
}
