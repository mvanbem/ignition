use crate::{ReadFrom, SerializedSize, WriteTo};
use std::convert::TryFrom;
use std::io::{self, Read, Write};
use std::marker::PhantomData;

/// A wrapper that owns a value and (de)serializes it with a size prefix.
///
/// # Example
///
/// ```
/// # use ignition_9p_wire::{OwnedSizePrefixed, ReadFrom};
/// # use ignition_9p_wire_derive::ReadFrom;
/// #[derive(Debug, ReadFrom, PartialEq)]
/// struct ExampleType {
///     field: u16,
/// }
///
/// let mut data: &'static [u8] = &[2, 0x55, 0xaa];
/// assert_eq!(
///     OwnedSizePrefixed::<u8, ExampleType>::read_from(&mut data)?.into_inner(),
///     ExampleType { field: 0xaa55 },
/// );
/// # Result::<(), std::io::Error>::Ok(())
/// ```
pub struct OwnedSizePrefixed<I, T> {
    value: T,
    _phantom_i: PhantomData<I>,
}
impl<I, T> OwnedSizePrefixed<I, T> {
    /// Wraps a value.
    pub fn new(value: T) -> Self {
        OwnedSizePrefixed {
            value,
            _phantom_i: PhantomData,
        }
    }
    /// Consumes self, returning the wrapped value.
    pub fn into_inner(self) -> T {
        self.value
    }
}
impl<I, T> ReadFrom for OwnedSizePrefixed<I, T>
where
    u64: TryFrom<I>,
    I: ReadFrom,
    T: ReadFrom,
{
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
        let size = u64::try_from(I::read_from(r)?).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "value too large to represent in memory",
            )
        })?;
        let mut r = Read::take(r.by_ref(), size);
        let value = T::read_from(&mut r)?;
        if r.limit() != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                // TODO: Break this out into a proper Error type and surface the specific numbers.
                "unread bytes after size-prefixed value",
            ));
        }
        Ok(OwnedSizePrefixed::new(value))
    }
}
impl<I, T> WriteTo for OwnedSizePrefixed<I, T>
where
    I: TryFrom<usize> + WriteTo,
    T: WriteTo + SerializedSize,
{
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        BorrowedSizePrefixed::<I, T>::new(&self.value).write_to(w)
    }
}

/// A wrapper that borrows a value and serializes it with a size prefix.
///
/// # Example
///
/// ```
/// # use ignition_9p_wire::{BorrowedSizePrefixed, SerializedSize, WriteTo};
/// # use ignition_9p_wire_derive::WriteTo;
/// #[derive(WriteTo)]
/// struct ExampleType {
///     field: u16,
/// }
/// impl SerializedSize for ExampleType {
///     fn serialized_size(&self) -> usize {
///         2
///     }
/// }
///
/// let mut data = vec![];
/// BorrowedSizePrefixed::<u8, _>::new(&ExampleType { field: 0xaa55 }).write_to(&mut data)?;
/// assert_eq!(
///     data.as_slice(),
///     &[2, 0x55, 0xaa],
/// );
/// # Result::<(), std::io::Error>::Ok(())
/// ```
pub struct BorrowedSizePrefixed<'a, I, T> {
    value: &'a T,
    _phantom_i: PhantomData<I>,
}
impl<'a, I, T> BorrowedSizePrefixed<'a, I, T> {
    /// Wraps a value.
    pub fn new(value: &'a T) -> Self {
        BorrowedSizePrefixed {
            value,
            _phantom_i: PhantomData,
        }
    }
}
impl<'a, I, T> WriteTo for BorrowedSizePrefixed<'a, I, T>
where
    I: TryFrom<usize> + WriteTo,
    T: WriteTo + SerializedSize,
{
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        let size = self.value.serialized_size();
        I::try_from(size)
            .map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidInput, "value too large to serailize")
            })?
            .write_to(w)?;
        let w = &mut LimitedWriter::new(w, size as u64);
        self.value.write_to(w)?;
        if w.limit() != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                // TODO: Break this out into a proper Error type and surface the specific numbers.
                "unwritten bytes after size-prefixed value",
            ));
        }
        Ok(())
    }
}

/// Writer adaptor which limits the bytes written to an underlying writer.
pub struct LimitedWriter<W: Write> {
    inner: W,
    limit: u64,
}
impl<W: Write> LimitedWriter<W> {
    /// Returns a new wrapper with the given underlying writer and byte limit.
    pub fn new(inner: W, limit: u64) -> Self {
        LimitedWriter { inner, limit }
    }
    /// Returns the byte limit.
    pub fn limit(&self) -> u64 {
        self.limit
    }
    /// Sets the byte limit.
    pub fn set_limit(&mut self, limit: u64) {
        self.limit = limit;
    }
    /// Consumes self, returning the underlying writer.
    pub fn into_inner(self) -> W {
        self.inner
    }
}
impl<W: Write> Write for LimitedWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.limit == 0 {
            return Ok(0);
        }

        let n = std::cmp::min(buf.len() as u64, self.limit);
        let n = self.inner.write(&buf[..n as usize])?;
        self.limit -= n as u64;
        Ok(n)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}
