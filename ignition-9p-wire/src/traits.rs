use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::convert::TryFrom;
use std::io::{self, Read, Write};

/// Types that can be deserialized from the 9p2000 wire protocol.
///
/// # Derivable
///
/// This trait can be used with `#[derive]` on structs or tuple structs if all fields are `ReadFrom`
/// or are [`Vec`]s or slices of `ReadFrom` elements.
///
/// ```
/// # use ignition_9p_wire_derive::ReadFrom;
/// #[derive(ReadFrom)]
/// struct Example {
///     field_a: u32,
///     field_b: String,
/// }
/// ```
///
/// The `derive`d implementation reads each field in declaration order.
///
/// ## Attributes
///
/// A number of options are available on the `#[ignition_9p_wire()]` attribute to customize the
/// behavior of a `derive`d implementation, adding collection counts and byte sizes on the wire. All
/// sizes are enforced during deserialization. Attempting to read past the end of a delimited region
/// or leaving any bytes unread at the end of a delimited region raises an error.
///
/// ### Struct prefixes
///
/// Structs may have an embedded size prefix on the wire.
///
/// ```
/// # use ignition_9p_wire::ReadFrom;
/// # use ignition_9p_wire_derive::ReadFrom;
/// #[derive(Debug, ReadFrom, PartialEq)]
/// #[ignition_9p_wire(embedded_size_prefix = "u32")]
/// struct Example {
///     field_a: u16,
///     field_b: u16,
/// }
///
/// let mut data: &'static [u8] = &[4, 0, 0, 0, 0x55, 0xaa, 0x34, 0x12];
/// assert_eq!(
///     Example::read_from(&mut data)?,
///     Example {
///         field_a: 0xaa55,
///         field_b: 0x1234,
///     },
/// );
/// # Result::<(), std::io::Error>::Ok(())
/// ```
///
/// ### Field prefixes
///
/// Scalar fields may be unprefixed or size-prefixed. [`Vec`] and slice fields are count-prefixed
/// or, for `u8` elements only, may be handled as length-prefixed bytes. Specifying multiple field
/// prefix options is an error.
///
/// ```
/// # use ignition_9p_wire::ReadFrom;
/// # use ignition_9p_wire_derive::ReadFrom;
/// #[derive(Debug, ReadFrom, PartialEq)]
/// struct Example {
///     unprefixed: u32,
///
///     #[ignition_9p_wire(size_prefixed = "u8")]
///     size_prefixed: u32,
///
///     #[ignition_9p_wire(count_prefixed = "u16")]
///     count_prefixed: Vec<u16>,
///
///     #[ignition_9p_wire(length_prefixed_bytes = "u32")]
///     length_prefixed_bytes: Vec<u8>,
/// }
///
/// let mut data: &'static [u8] = &[
///     // unprefixed field
///     0x78, 0x56, 0x34, 0x12,
///
///     // u8-size-prefixed field
///     4, 0x78, 0x56, 0x34, 0x12,
///
///     // u16-count-prefixed field
///     3, 0, 0x34, 0x12, 0x78, 0x56, 0xbc, 0x9a,
///
///     // u32-length-prefixed bytes field
///     8, 0, 0, 0, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
/// ];
/// assert_eq!(
///     Example::read_from(&mut data)?,
///     Example {
///         unprefixed: 0x12345678,
///         size_prefixed: 0x12345678,
///         count_prefixed: vec![0x1234, 0x5678, 0x9abc],
///         length_prefixed_bytes: vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef],
///     },
/// );
/// # Result::<(), std::io::Error>::Ok(())
/// ```
pub trait ReadFrom: Sized {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self>;
}

/// Types that can be serialized to the 9p2000 wire protocol.
///
/// # Derivable
///
/// This trait can be used with `#[derive]` on structs or tuple structs if all fields are `WriteTo`
/// or are [`Vec`]s or slices of `WriteTo` elements.
///
/// ```
/// # use ignition_9p_wire_derive::WriteTo;
/// #[derive(WriteTo)]
/// struct Example {
///     field_a: u32,
///     field_b: String,
/// }
/// ```
///
/// The `derive`d implementation writes each field in declaration order.
///
/// ## Attributes
///
/// A number of options are available on the `#[ignition_9p_wire()]` attribute to customize the
/// behavior of a `derive`d implementation, adding collection counts and byte sizes on the wire. All
/// sizes are enforced during serialization. Attempting to write past the end of a delimited region
/// or leaving a delimited region less than fully written raises an error.
///
/// ### Struct prefixes
///
/// Structs may have an embedded size prefix on the wire. The struct must implement
/// [`EmbeddedSize`].
///
/// ```
/// # use ignition_9p_wire::{EmbeddedSize, WriteTo};
/// # use ignition_9p_wire_derive::WriteTo;
/// #[derive(WriteTo)]
/// #[ignition_9p_wire(embedded_size_prefix = "u32")]
/// struct Example {
///     field_a: u16,
///     field_b: u16,
/// }
/// impl EmbeddedSize for Example {
///     fn embedded_size(&self) -> usize {
///         4
///     }
/// }
///
/// let mut data = vec![];
/// Example {
///     field_a: 0xaa55,
///     field_b: 0x1234,
/// }.write_to(&mut data)?;
/// assert_eq!(data.as_slice(), &[4, 0, 0, 0, 0x55, 0xaa, 0x34, 0x12]);
/// # Result::<(), std::io::Error>::Ok(())
/// ```
///
/// ### Field prefixes
///
/// Scalar fields may be unprefixed or size-prefixed. [`Vec`] and slice fields are count-prefixed
/// or, for `u8` elements only, may be handled as length-prefixed bytes. Specifying multiple field
/// prefix options is an error.
///
/// Size-prefixed fields must implement [`SerializedSize`].
///
/// ```
/// # use ignition_9p_wire::{SerializedSize, WriteTo};
/// # use ignition_9p_wire_derive::WriteTo;
/// #[derive(WriteTo)]
/// struct Example {
///     unprefixed: u32,
///
///     #[ignition_9p_wire(size_prefixed = "u8")]
///     size_prefixed: SizedU32,
///
///     #[ignition_9p_wire(count_prefixed = "u16")]
///     count_prefixed: Vec<u16>,
///
///     #[ignition_9p_wire(length_prefixed_bytes = "u32")]
///     length_prefixed_bytes: Vec<u8>,
/// }
/// #[derive(WriteTo)]
/// struct SizedU32(u32);
/// impl SerializedSize for SizedU32 {
///     fn serialized_size(&self) -> usize {
///         4
///     }
/// }
///
/// let mut data = vec![];
/// Example {
///     unprefixed: 0x12345678,
///     size_prefixed: SizedU32(0x12345678),
///     count_prefixed: vec![0x1234, 0x5678, 0x9abc],
///     length_prefixed_bytes: vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef],
/// }.write_to(&mut data)?;
/// assert_eq!(
///     data.as_slice(),
///     &[
///         // unprefixed field
///         0x78, 0x56, 0x34, 0x12,
///
///         // u8-size-prefixed field
///         4, 0x78, 0x56, 0x34, 0x12,
///
///         // u16-count-prefixed field
///         3, 0, 0x34, 0x12, 0x78, 0x56, 0xbc, 0x9a,
///
///         // u32-length-prefixed bytes field
///         8, 0, 0, 0, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
///     ],
/// );
/// # Result::<(), std::io::Error>::Ok(())
/// ```
pub trait WriteTo {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()>;
}

/// Types that know their serialized size.
///
/// `Derive`d implementations of [`WriteTo`] serialize the provided size first and then enforce that
/// the rest of the value writes precisely that number of bytes.
pub trait SerializedSize {
    fn serialized_size(&self) -> usize;
}

/// Types that embed their size as a prefix of their wire format.
///
/// The motivating example is the 9p `stat` struct, which embeds the length of its contents,
/// excluding the size field itself. This trait allows `stat` to avoid declaring a size field and
/// asking all users to fill it in accurately, having it automated instead.
///
/// Use this trait together with the `embedded_size_prefix` attribute key when deriving [`ReadFrom`]
/// or [`WriteTo`]:
///
/// ```
/// # use ignition_9p_wire::{EmbeddedSize, ReadFrom, WriteTo};
/// # use ignition_9p_wire_derive::{ReadFrom, WriteTo};
/// #[derive(Debug, ReadFrom, WriteTo, PartialEq)]
/// #[ignition_9p_wire(embedded_size_prefix = "u16")]
/// struct ExampleType {
///     field_a: u32,
/// }
/// impl EmbeddedSize for ExampleType {
///     fn embedded_size(&self) -> usize {
///         4
///     }
/// }
///
/// let mut buf: &'static [u8] = &[4, 0, 0x78, 0x56, 0x34, 0x12];
/// assert_eq!(
///     ExampleType::read_from(&mut buf)?,
///     ExampleType { field_a: 0x12345678 },
/// );
/// # Result::<(), std::io::Error>::Ok(())
/// ```
pub trait EmbeddedSize {
    fn embedded_size(&self) -> usize;
}

impl ReadFrom for u8 {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
        r.read_u8()
    }
}
impl WriteTo for u8 {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        w.write_u8(*self)
    }
}

impl ReadFrom for u16 {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
        r.read_u16::<LittleEndian>()
    }
}
impl WriteTo for u16 {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        w.write_u16::<LittleEndian>(*self)
    }
}

impl ReadFrom for u32 {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
        r.read_u32::<LittleEndian>()
    }
}
impl WriteTo for u32 {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        w.write_u32::<LittleEndian>(*self)
    }
}

impl ReadFrom for u64 {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
        r.read_u64::<LittleEndian>()
    }
}
impl WriteTo for u64 {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        w.write_u64::<LittleEndian>(*self)
    }
}

impl ReadFrom for String {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
        let len = u16::read_from(r)? as usize;
        let mut buf = vec![0; len];
        r.read_exact(buf.as_mut_slice())?;
        String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}
impl WriteTo for String {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        self.as_str().write_to(w)
    }
}
impl WriteTo for &str {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        u16::try_from(self.len())
            .map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidInput, "value too large to serailize")
            })?
            .write_to(w)?;
        w.write_all(self.as_bytes())
    }
}
