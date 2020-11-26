use crate::ext::{ReadBytesExt, WriteBytesExt};
use std::io;

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

    pub fn read<R: io::Read>(r: &mut R) -> io::Result<Tag> {
        Ok(Tag(r.read_u16()?))
    }

    pub fn write<W: io::Write>(self, w: &mut W) -> io::Result<()> {
        w.write_u16(self.0)
    }
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
