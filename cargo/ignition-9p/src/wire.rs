use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::convert::TryFrom;
use std::io::{self, Read, Write};
use std::marker::PhantomData;

pub trait ReadWireFormat {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self>
    where
        Self: Sized;
}
pub trait WriteWireFormat {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()>;
}

impl ReadWireFormat for u8 {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
        r.read_u8()
    }
}
impl WriteWireFormat for u8 {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        w.write_u8(*self)
    }
}

impl ReadWireFormat for u16 {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
        r.read_u16::<LittleEndian>()
    }
}
impl WriteWireFormat for u16 {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        w.write_u16::<LittleEndian>(*self)
    }
}

impl ReadWireFormat for u32 {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
        r.read_u32::<LittleEndian>()
    }
}
impl WriteWireFormat for u32 {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        w.write_u32::<LittleEndian>(*self)
    }
}

impl ReadWireFormat for u64 {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
        r.read_u64::<LittleEndian>()
    }
}
impl WriteWireFormat for u64 {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        w.write_u64::<LittleEndian>(*self)
    }
}

impl ReadWireFormat for String {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
        let len = u16::read_from(r)? as usize;
        let mut buf = vec![0; len];
        r.read_exact(buf.as_mut_slice())?;
        String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}
impl WriteWireFormat for String {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        self.as_str().write_to(w)
    }
}
impl WriteWireFormat for &str {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        u16::try_from(self.len())
            .map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidInput, "value too large to serailize")
            })?
            .write_to(w)?;
        w.write_all(self.as_bytes())
    }
}

pub struct OwnedCountPrefixedList<I, T> {
    vec: Vec<T>,
    _phantom_i: PhantomData<I>,
}
impl<I, T> OwnedCountPrefixedList<I, T> {
    pub fn new(vec: Vec<T>) -> Self {
        OwnedCountPrefixedList {
            vec,
            _phantom_i: PhantomData,
        }
    }
    pub fn into_inner(self) -> Vec<T> {
        self.vec
    }
}
impl<I, T> ReadWireFormat for OwnedCountPrefixedList<I, T>
where
    usize: TryFrom<I>,
    I: ReadWireFormat,
    T: ReadWireFormat,
{
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
        let count = usize::try_from(I::read_from(r)?).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "value too large to represent in memory",
            )
        })?;
        let mut vec = vec![];
        for _ in 0..count {
            vec.push(T::read_from(r)?);
        }
        Ok(OwnedCountPrefixedList::new(vec))
    }
}
impl<I, T> WriteWireFormat for OwnedCountPrefixedList<I, T>
where
    I: TryFrom<usize> + WriteWireFormat,
    T: WriteWireFormat,
{
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        BorrowedCountPrefixedList::<I, T>::new(&self.vec).write_to(w)
    }
}

pub struct BorrowedCountPrefixedList<'a, I, T> {
    slice: &'a [T],
    _phantom_i: PhantomData<I>,
}
impl<'a, I, T> BorrowedCountPrefixedList<'a, I, T> {
    pub fn new(slice: &'a [T]) -> Self {
        BorrowedCountPrefixedList {
            slice,
            _phantom_i: PhantomData,
        }
    }
}
impl<'a, I, T> WriteWireFormat for BorrowedCountPrefixedList<'a, I, T>
where
    I: TryFrom<usize> + WriteWireFormat,
    T: WriteWireFormat,
{
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        I::try_from(self.slice.len())
            .map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidInput, "value too large to serailize")
            })?
            .write_to(w)?;
        for item in self.slice {
            item.write_to(w)?;
        }
        Ok(())
    }
}

pub trait SerializedSize {
    fn serialized_size(&self) -> usize;
}

pub struct OwnedLengthPrefixed<I, T> {
    value: T,
    _phantom_i: PhantomData<I>,
}
impl<I, T> OwnedLengthPrefixed<I, T> {
    pub fn new(value: T) -> Self {
        OwnedLengthPrefixed {
            value,
            _phantom_i: PhantomData,
        }
    }
    pub fn into_inner(self) -> T {
        self.value
    }
}
impl<I, T> ReadWireFormat for OwnedLengthPrefixed<I, T>
where
    u64: TryFrom<I>,
    I: ReadWireFormat,
    T: ReadWireFormat,
{
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
        let size = u64::try_from(I::read_from(r)?).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "value too large to represent in memory",
            )
        })?;
        let mut r = r.by_ref().take(size);
        let value = T::read_from(&mut r)?;
        if r.limit() != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                // TODO: Break this out into a proper Error type and surface the specific numbers.
                "unread bytes after length-prefixed value",
            ));
        }
        Ok(OwnedLengthPrefixed::new(value))
    }
}
impl<I, T> WriteWireFormat for OwnedLengthPrefixed<I, T>
where
    I: TryFrom<usize> + WriteWireFormat,
    T: WriteWireFormat + SerializedSize,
{
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        BorrowedLengthPrefixed::<I, T>::new(&self.value).write_to(w)
    }
}

pub struct BorrowedLengthPrefixed<'a, I, T> {
    value: &'a T,
    _phantom_i: PhantomData<I>,
}
impl<'a, I, T> BorrowedLengthPrefixed<'a, I, T> {
    pub fn new(value: &'a T) -> Self {
        BorrowedLengthPrefixed {
            value,
            _phantom_i: PhantomData,
        }
    }
}
impl<'a, I, T> WriteWireFormat for BorrowedLengthPrefixed<'a, I, T>
where
    I: TryFrom<usize> + WriteWireFormat,
    T: WriteWireFormat + SerializedSize,
{
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        I::try_from(self.value.serialized_size())
            .map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidInput, "value too large to serailize")
            })?
            .write_to(w)?;
        self.value.write_to(w)
    }
}
