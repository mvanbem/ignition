use crate::wire::{ReadWireFormat, WriteWireFormat};
use derive_more::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign};
use std::io::{self, Read, Write};

bitfield::bitfield! {
    /// A mode for opening and creating files.
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
    pub struct OpenMode(u8);
    impl Debug;
    pub from into OpenAccess, access, set_access: 1, 0;
    pub trunc, set_trunc: 4;
    pub rclose, set_rclose: 6;
}
impl OpenMode {
    pub const TRUNC: OpenMode = OpenMode(0x10);
    pub const RCLOSE: OpenMode = OpenMode(0x40);

    pub fn map_access<F>(self, f: F) -> Self
    where
        F: FnOnce(OpenAccess) -> OpenAccess,
    {
        self.with_access(f(self.access()))
    }
    pub fn with_access(mut self, access: OpenAccess) -> Self {
        self.set_access(access);
        self
    }
    pub fn with_trunc(mut self, trunc: bool) -> Self {
        self.set_trunc(trunc);
        self
    }
    pub fn with_rclose(mut self, rclose: bool) -> Self {
        self.set_rclose(rclose);
        self
    }
}
impl From<OpenMode> for u8 {
    fn from(mode: OpenMode) -> Self {
        mode.0
    }
}
impl From<u8> for OpenMode {
    fn from(value: u8) -> Self {
        OpenMode(value)
    }
}
impl ReadWireFormat for OpenMode {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
        Ok(OpenMode(ReadWireFormat::read_from(r)?))
    }
}
impl WriteWireFormat for OpenMode {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        self.0.write_to(w)
    }
}

/// An access mode for opening and creating files.
///
/// The value is stored in the least significant two bits of a [`u8`]. The other six bits are
/// ignored in the implementations of [`Debug`] and [`PartialEq`].
#[derive(BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Clone, Copy, Eq)]
pub struct OpenAccess(pub u8);
impl OpenAccess {
    /// Bit mask precisely covering the meaningful bits of an open access value.
    ///
    /// The expression `x & Access::MASK == x` evaluates true for any `Access` value x.
    pub const MASK: OpenAccess = OpenAccess(0x03);

    /// Read access.
    pub const READ: OpenAccess = OpenAccess(0);
    /// Write access.
    pub const WRITE: OpenAccess = OpenAccess(1);
    /// Read and write access.
    pub const RDWR: OpenAccess = OpenAccess(2);
    /// Execute access.
    pub const EXEC: OpenAccess = OpenAccess(3);
}
impl std::fmt::Debug for OpenAccess {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match (*self & OpenAccess::MASK).0 {
                0 => "OREAD",
                1 => "OWRITE",
                2 => "ORDWR",
                3 => "OEXEC",
                _ => unreachable!(),
            },
        )
    }
}
impl From<OpenAccess> for u8 {
    fn from(access: OpenAccess) -> Self {
        access.0
    }
}
impl From<u8> for OpenAccess {
    fn from(value: u8) -> Self {
        OpenAccess(value)
    }
}
impl PartialEq for OpenAccess {
    fn eq(&self, other: &Self) -> bool {
        (*self & OpenAccess::MASK).0 == (*other & OpenAccess::MASK).0
    }
}

#[cfg(test)]
mod tests {
    use super::{OpenAccess, OpenMode};

    #[test]
    fn access_constants() {
        assert_eq!(OpenMode(0x00).access(), OpenAccess::READ);
        assert_eq!(OpenMode(0xfc).access(), OpenAccess::READ);
        assert_eq!(OpenMode(0x01).access(), OpenAccess::WRITE);
        assert_eq!(OpenMode(0xfd).access(), OpenAccess::WRITE);
        assert_eq!(OpenMode(0x02).access(), OpenAccess::RDWR);
        assert_eq!(OpenMode(0xfe).access(), OpenAccess::RDWR);
        assert_eq!(OpenMode(0x03).access(), OpenAccess::EXEC);
        assert_eq!(OpenMode(0xff).access(), OpenAccess::EXEC);
    }

    #[test]
    fn mode_constants() {
        assert_eq!(OpenMode::TRUNC.trunc(), true);
        assert_eq!(OpenMode::RCLOSE.rclose(), true);
    }
}
