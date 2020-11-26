//! Extension traits for serializing and deserializing basic types in the 9P2000 wire format.

use std::convert::TryInto;
use std::io::{self, Read, Write};

type Endian = byteorder::LittleEndian;

/// [`Read`] extensions for the 9P2000 wire format.
///
/// All integers are ordered in little endian.
pub trait ReadBytesExt: Read {
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
    fn read_u16_prefixed_bytes(&mut self) -> io::Result<Vec<u8>> {
        let size = self.read_u16()?;
        let mut buf = Vec::new();
        buf.resize(size.into(), 0);
        self.read_exact(buf.as_mut_slice())?;
        Ok(buf)
    }
    fn read_u32_prefixed_bytes(&mut self) -> io::Result<Vec<u8>> {
        let size = self.read_u32()?;
        let mut buf = Vec::new();
        buf.resize(size as usize, 0);
        self.read_exact(buf.as_mut_slice())?;
        Ok(buf)
    }
    fn read_string(&mut self) -> io::Result<String> {
        String::from_utf8(self.read_u16_prefixed_bytes()?)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
    fn read_array<T, F>(&mut self, f: F) -> io::Result<Vec<T>>
    where
        F: Fn(&mut Self) -> io::Result<T>,
    {
        let count = self.read_u16()? as usize;
        let mut items = Vec::with_capacity(count);
        for _ in 0..count {
            items.push(f(self)?);
        }
        Ok(items)
    }
}
impl<T> ReadBytesExt for T where T: Read {}

/// [`Write`] extensions for the 9P2000 wire format.
///
/// All integers are ordered in little endian.
pub trait WriteBytesExt: Write {
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
    fn write_u16_prefixed_bytes(&mut self, value: &[u8]) -> io::Result<()> {
        self.write_u16(
            value
                .len()
                .try_into()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?,
        )?;
        self.write_all(value)?;
        Ok(())
    }
    fn write_u32_prefixed_bytes(&mut self, value: &[u8]) -> io::Result<()> {
        self.write_u32(
            value
                .len()
                .try_into()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?,
        )?;
        self.write_all(value)?;
        Ok(())
    }
    fn write_string(&mut self, value: &str) -> io::Result<()> {
        self.write_u16_prefixed_bytes(value.as_bytes())
    }
    fn write_array<T, F>(&mut self, items: &[T], f: F) -> io::Result<()>
    where
        F: Fn(&mut Self, &T) -> io::Result<()>,
    {
        self.write_u16(
            items
                .len()
                .try_into()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?,
        )?;
        for item in items {
            f(self, item)?;
        }
        Ok(())
    }
}
impl<T> WriteBytesExt for T where T: Write {}
