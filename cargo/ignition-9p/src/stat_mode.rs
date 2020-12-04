use crate::wire::{ReadWireFormat, WriteWireFormat};
use crate::{DontTouch, FileType, UnixTriplet};
use derive_more::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign};
use std::io::{self, Read, Write};

bitfield::bitfield! {
    /// A combined mode type as used in [`Stat::mode`](crate::Stat::mode).
    ///
    /// Embeds a [`FileType`] as well as the nine Unix file permission bits.
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
    pub struct StatMode(u32);
    impl Debug;
    pub from into FileType, file_type, set_file_type: 31, 24;
    pub from into UnixTriplet, user, set_user: 8, 6;
    pub from into UnixTriplet, group, set_group: 5, 3;
    pub from into UnixTriplet, other, set_other: 2, 0;
}
impl StatMode {
    pub fn map_file_type<F>(self, f: F) -> Self
    where
        F: FnOnce(FileType) -> FileType,
    {
        self.with_file_type(f(self.file_type()))
    }
    pub fn with_file_type(mut self, file_type: FileType) -> Self {
        self.set_file_type(file_type);
        self
    }
    pub fn with_user(mut self, user: UnixTriplet) -> Self {
        self.set_user(user);
        self
    }
    pub fn with_group(mut self, group: UnixTriplet) -> Self {
        self.set_group(group);
        self
    }
    pub fn with_other(mut self, other: UnixTriplet) -> Self {
        self.set_other(other);
        self
    }
}
impl DontTouch for StatMode {
    fn dont_touch() -> Self {
        StatMode(!0)
    }
}
impl From<StatMode> for u32 {
    fn from(mode: StatMode) -> Self {
        mode.0
    }
}
impl From<u32> for StatMode {
    fn from(value: u32) -> Self {
        StatMode(value)
    }
}
impl ReadWireFormat for StatMode {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
        Ok(StatMode(ReadWireFormat::read_from(r)?))
    }
}
impl WriteWireFormat for StatMode {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        self.0.write_to(w)
    }
}

#[cfg(test)]
mod tests {
    use super::StatMode;

    #[test]
    fn bits() {
        assert_eq!(StatMode(0x80000000).file_type().dir(), true);
        assert_eq!(StatMode(0x40000000).file_type().append(), true);
        assert_eq!(StatMode(0x20000000).file_type().excl(), true);
        assert_eq!(StatMode(0x08000000).file_type().auth(), true);
        assert_eq!(StatMode(0x04000000).file_type().tmp(), true);
        assert_eq!(StatMode(0x00000100).user().read(), true);
        assert_eq!(StatMode(0x00000080).user().write(), true);
        assert_eq!(StatMode(0x00000040).user().execute(), true);
        assert_eq!(StatMode(0x00000020).group().read(), true);
        assert_eq!(StatMode(0x00000010).group().write(), true);
        assert_eq!(StatMode(0x00000008).group().execute(), true);
        assert_eq!(StatMode(0x00000004).other().read(), true);
        assert_eq!(StatMode(0x00000002).other().write(), true);
        assert_eq!(StatMode(0x00000001).other().execute(), true);
    }
}
