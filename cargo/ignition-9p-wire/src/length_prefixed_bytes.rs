use crate::{ReadFrom, WriteTo};
use std::convert::TryFrom;
use std::io::{self, Read, Write};
use std::marker::PhantomData;

/// A wrapper that owns a [`Vec`] of bytes and (de)serializes them with a length prefix.
///
/// # Example
///
/// ```
/// # use ignition_9p_wire::{OwnedLengthPrefixedBytes, ReadFrom};
/// let mut data: &'static [u8] = &[4, 0, 0, 0, 0x12, 0x34, 0x56, 0x78];
/// assert_eq!(
///     OwnedLengthPrefixedBytes::<u32>::read_from(&mut data)?.into_inner().as_slice(),
///     &[0x12, 0x34, 0x56, 0x78],
/// );
/// # Result::<(), std::io::Error>::Ok(())
/// ```
pub struct OwnedLengthPrefixedBytes<I> {
    vec: Vec<u8>,
    _phantom_i: PhantomData<I>,
}
impl<I> OwnedLengthPrefixedBytes<I> {
    /// Wraps a value.
    pub fn new(vec: Vec<u8>) -> Self {
        OwnedLengthPrefixedBytes {
            vec,
            _phantom_i: PhantomData,
        }
    }
    /// Consumes self, returning the wrapped value.
    pub fn into_inner(self) -> Vec<u8> {
        self.vec
    }
}
impl<I> ReadFrom for OwnedLengthPrefixedBytes<I>
where
    usize: TryFrom<I>,
    I: ReadFrom,
{
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
        let count = usize::try_from(I::read_from(r)?).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "value too large to represent in memory",
            )
        })?;
        // TODO: Consider using unsafe code to avoid initializing the buffer.
        let mut vec = vec![0; count];
        r.read_exact(&mut vec)?;
        Ok(OwnedLengthPrefixedBytes::new(vec))
    }
}
impl<I> WriteTo for OwnedLengthPrefixedBytes<I>
where
    I: TryFrom<usize> + WriteTo,
{
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        BorrowedLengthPrefixedBytes::<I>::new(&self.vec).write_to(w)
    }
}

/// A wrapper that borrows a slice of bytes and serializes them with a length prefix.
///
/// # Example
///
/// ```
/// # use ignition_9p_wire::{BorrowedLengthPrefixedBytes, WriteTo};
/// let mut data = vec![];
/// BorrowedLengthPrefixedBytes::<u32>::new(&[0x12, 0x34, 0x56, 0x78]).write_to(&mut data)?;
/// assert_eq!(
///     data.as_slice(),
///     &[4, 0, 0, 0, 0x12, 0x34, 0x56, 0x78],
/// );
/// # Result::<(), std::io::Error>::Ok(())
/// ```
pub struct BorrowedLengthPrefixedBytes<'a, I> {
    slice: &'a [u8],
    _phantom_i: PhantomData<I>,
}
impl<'a, I> BorrowedLengthPrefixedBytes<'a, I> {
    /// Wraps a value.
    pub fn new(slice: &'a [u8]) -> Self {
        BorrowedLengthPrefixedBytes {
            slice,
            _phantom_i: PhantomData,
        }
    }
}
impl<'a, I> WriteTo for BorrowedLengthPrefixedBytes<'a, I>
where
    I: TryFrom<usize> + WriteTo,
{
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        I::try_from(self.slice.len())
            .map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidInput, "value too large to serailize")
            })?
            .write_to(w)?;
        w.write_all(self.slice)
    }
}
