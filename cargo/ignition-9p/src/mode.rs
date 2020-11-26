#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Mode(u8);
impl Mode {
    const OEXCL: u8 = 0x04;
    const OTRUNC: u8 = 0x10;
    const ORCLOSE: u8 = 0x40;
    const OAPPEND: u8 = 0x80;

    pub fn access(self) -> Access {
        Access(self.0 & Access::MASK)
    }
    pub fn excl(self) -> bool {
        (self.0 & Mode::OEXCL) != 0
    }
    pub fn trunc(self) -> bool {
        (self.0 & Mode::OTRUNC) != 0
    }
    pub fn rclose(self) -> bool {
        (self.0 & Mode::ORCLOSE) != 0
    }
    pub fn append(self) -> bool {
        (self.0 & Mode::OAPPEND) != 0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Access(u8);
impl Access {
    const MASK: u8 = 0x03;

    pub const READ: Access = Access(0x00);
    pub const WRITE: Access = Access(0x01);
    pub const RDWR: Access = Access(0x02);
    pub const EXEC: Access = Access(0x03);
}
