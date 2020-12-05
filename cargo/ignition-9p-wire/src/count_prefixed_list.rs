use crate::{ReadFrom, WriteTo};
use std::convert::TryFrom;
use std::io::{self, Read, Write};
use std::marker::PhantomData;

/// A wrapper that owns a [`Vec`] of values and (de)serializes them with a count prefix.
///
/// # Efficiency note
///
/// For potentially large `Vec<u8>`, prefer
/// [`OwnedLengthPrefixedBytes`](crate::length_prefixed_bytes::OwnedLengthPrefixedBytes), which will
/// make fewer calls to the underlying [`Read`] or [`Write`].
///
/// # Example
///
/// ```
/// # use ignition_9p_wire::{OwnedCountPrefixedList, ReadFrom};
/// # use ignition_9p_wire_derive::ReadFrom;
/// #[derive(Debug, ReadFrom, PartialEq)]
/// struct ExampleType(u16);
///
/// let mut data: &'static [u8] = &[2, 0x55, 0xaa, 0x34, 0x12];
/// assert_eq!(
///     OwnedCountPrefixedList::<u8, ExampleType>::read_from(&mut data)?.into_inner().as_slice(),
///     &[ExampleType(0xaa55), ExampleType(0x1234)],
/// );
/// # Result::<(), std::io::Error>::Ok(())
/// ```
pub struct OwnedCountPrefixedList<I, T> {
    vec: Vec<T>,
    _phantom_i: PhantomData<I>,
}
impl<I, T> OwnedCountPrefixedList<I, T> {
    /// Wraps a value.
    pub fn new(vec: Vec<T>) -> Self {
        OwnedCountPrefixedList {
            vec,
            _phantom_i: PhantomData,
        }
    }
    /// Consumes self, returning the wrapped value.
    pub fn into_inner(self) -> Vec<T> {
        self.vec
    }
}
impl<I, T> ReadFrom for OwnedCountPrefixedList<I, T>
where
    usize: TryFrom<I>,
    I: ReadFrom,
    T: ReadFrom,
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
impl<I, T> WriteTo for OwnedCountPrefixedList<I, T>
where
    I: TryFrom<usize> + WriteTo,
    T: WriteTo,
{
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        BorrowedCountPrefixedList::<I, T>::new(&self.vec).write_to(w)
    }
}

/// A wrapper that borrows a slice of values and serializes them with a count prefix.
///
/// # Efficiency note
///
/// For potentially large `Vec<u8>`, prefer
/// [`BorrowedLengthPrefixedBytes`](crate::length_prefixed_bytes::BorrowedLengthPrefixedBytes),
/// which will make fewer calls to the underlying [`Write`].
///
/// # Example
///
/// ```
/// # use ignition_9p_wire::{BorrowedCountPrefixedList, WriteTo};
/// # use ignition_9p_wire_derive::WriteTo;
/// #[derive(WriteTo)]
/// struct ExampleType(u16);
///
/// let mut data = vec![];
/// BorrowedCountPrefixedList::<u8, _>::new(&[ExampleType(0xaa55), ExampleType(0x1234)])
///     .write_to(&mut data)?;
/// assert_eq!(
///     data.as_slice(),
///     &[2, 0x55, 0xaa, 0x34, 0x12],
/// );
/// # Result::<(), std::io::Error>::Ok(())
/// ```
pub struct BorrowedCountPrefixedList<'a, I, T> {
    slice: &'a [T],
    _phantom_i: PhantomData<I>,
}
impl<'a, I, T> BorrowedCountPrefixedList<'a, I, T> {
    /// Wraps a value.
    pub fn new(slice: &'a [T]) -> Self {
        BorrowedCountPrefixedList {
            slice,
            _phantom_i: PhantomData,
        }
    }
}
impl<'a, I, T> WriteTo for BorrowedCountPrefixedList<'a, I, T>
where
    I: TryFrom<usize> + WriteTo,
    T: WriteTo,
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
