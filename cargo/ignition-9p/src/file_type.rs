use derive_more::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign};

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
    )]
    pub struct FileType(u8);
    impl Debug;
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
