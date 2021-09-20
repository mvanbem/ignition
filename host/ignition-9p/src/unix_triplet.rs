use derive_more::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign};

bitfield::bitfield! {
    /// Three Unix permission bits: read, write, and execute.
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
    pub struct UnixTriplet(u8);
    pub read, set_read: 2;
    pub write, set_write: 1;
    pub execute, set_execute: 0;
}
impl UnixTriplet {
    pub const NONE: UnixTriplet = UnixTriplet(0x0);
    pub const X: UnixTriplet = UnixTriplet(0x1);
    pub const W: UnixTriplet = UnixTriplet(0x2);
    pub const WX: UnixTriplet = UnixTriplet(0x3);
    pub const R: UnixTriplet = UnixTriplet(0x4);
    pub const RX: UnixTriplet = UnixTriplet(0x5);
    pub const RW: UnixTriplet = UnixTriplet(0x6);
    pub const RWX: UnixTriplet = UnixTriplet(0x7);

    pub fn with_read(mut self, read: bool) -> Self {
        self.set_read(read);
        self
    }
    pub fn with_write(mut self, write: bool) -> Self {
        self.set_write(write);
        self
    }
    pub fn with_execute(mut self, execute: bool) -> Self {
        self.set_execute(execute);
        self
    }
}
impl std::fmt::Debug for UnixTriplet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.read() {
            write!(f, "r")?;
        } else {
            write!(f, "-")?;
        }
        if self.write() {
            write!(f, "w")?;
        } else {
            write!(f, "-")?;
        }
        if self.execute() {
            write!(f, "x")?;
        } else {
            write!(f, "-")?;
        }
        Ok(())
    }
}
impl From<UnixTriplet> for u8 {
    fn from(mode: UnixTriplet) -> Self {
        mode.0
    }
}
impl From<UnixTriplet> for u32 {
    fn from(mode: UnixTriplet) -> Self {
        mode.0 as u32
    }
}
impl From<u8> for UnixTriplet {
    fn from(value: u8) -> Self {
        UnixTriplet(value)
    }
}
impl From<u32> for UnixTriplet {
    fn from(value: u32) -> Self {
        UnixTriplet(value as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::UnixTriplet;

    #[test]
    fn constants() {
        assert_eq!(UnixTriplet::NONE.read(), false);
        assert_eq!(UnixTriplet::NONE.write(), false);
        assert_eq!(UnixTriplet::NONE.execute(), false);
        assert_eq!(UnixTriplet::X.read(), false);
        assert_eq!(UnixTriplet::X.write(), false);
        assert_eq!(UnixTriplet::X.execute(), true);
        assert_eq!(UnixTriplet::W.read(), false);
        assert_eq!(UnixTriplet::W.write(), true);
        assert_eq!(UnixTriplet::W.execute(), false);
        assert_eq!(UnixTriplet::WX.read(), false);
        assert_eq!(UnixTriplet::WX.write(), true);
        assert_eq!(UnixTriplet::WX.execute(), true);
        assert_eq!(UnixTriplet::R.read(), true);
        assert_eq!(UnixTriplet::R.write(), false);
        assert_eq!(UnixTriplet::R.execute(), false);
        assert_eq!(UnixTriplet::RX.read(), true);
        assert_eq!(UnixTriplet::RX.write(), false);
        assert_eq!(UnixTriplet::RX.execute(), true);
        assert_eq!(UnixTriplet::RW.read(), true);
        assert_eq!(UnixTriplet::RW.write(), true);
        assert_eq!(UnixTriplet::RW.execute(), false);
        assert_eq!(UnixTriplet::RWX.read(), true);
        assert_eq!(UnixTriplet::RWX.write(), true);
        assert_eq!(UnixTriplet::RWX.execute(), true);
    }
}
