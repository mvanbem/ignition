use std::io;

type Endian = byteorder::LittleEndian;

pub trait ReadBytesExt: io::Read {
    fn read_u8(&mut self) -> io::Result<u8> {
        byteorder::ReadBytesExt::read_u8(self)
    }
    fn read_u16(&mut self) -> io::Result<u16> {
        byteorder::ReadBytesExt::read_u16::<Endian>(self)
    }
    fn read_u32(&mut self) -> io::Result<u32> {
        byteorder::ReadBytesExt::read_u32::<Endian>(self)
    }
    fn read_u64(&mut self) -> io::Result<u64> {
        byteorder::ReadBytesExt::read_u64::<Endian>(self)
    }
}
impl<T> ReadBytesExt for T where T: io::Read {}

pub trait WriteBytesExt: io::Write {
    fn write_u8(&mut self, value: u8) -> io::Result<()> {
        byteorder::WriteBytesExt::write_u8(self, value)
    }
    fn write_u16(&mut self, value: u16) -> io::Result<()> {
        byteorder::WriteBytesExt::write_u16::<Endian>(self, value)
    }
    fn write_u32(&mut self, value: u32) -> io::Result<()> {
        byteorder::WriteBytesExt::write_u32::<Endian>(self, value)
    }
    fn write_u64(&mut self, value: u64) -> io::Result<()> {
        byteorder::WriteBytesExt::write_u64::<Endian>(self, value)
    }
}
impl<T> WriteBytesExt for T where T: io::Write {}
