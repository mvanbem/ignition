use crate::wire::{ReadWireFormat, WriteWireFormat};
use std::io::{self, Read, Write};

/// A per-connection message identifier.
///
/// Each T-message has a tag field, chosen and used by the client to identify the message. The reply
/// to the message will have the same tag. Clients must arrange that no two outstanding messages on
/// the same connection have the same tag. An exception is the tag [`NOTAG`](Tag::NOTAG), defined as
///  `!0`: the client can use it when establishing a connection to override tag matching in version
/// messages.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Tag(pub u16);
impl Tag {
    /// A client may use `NOTAG` when establishing a connection to override tag matching in version
    /// messages.
    pub const NOTAG: Tag = Tag(!0);
}
impl std::fmt::Debug for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 == !0 {
            write!(f, "NOTAG")
        } else {
            write!(f, "{}", self.0)
        }
    }
}
impl ReadWireFormat for Tag {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
        Ok(Tag(ReadWireFormat::read_from(r)?))
    }
}
impl WriteWireFormat for Tag {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        self.0.write_to(w)
    }
}
