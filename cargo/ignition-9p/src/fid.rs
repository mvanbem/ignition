use crate::ext::{ReadBytesExt, WriteBytesExt};
use std::io;

/// A client's file identifier.
///
/// Most T-messages contain a fid. Fids are somewhat like file descriptors in a user process, but
/// they are not restricted to files open for I/O: directories being examined, files being accessed
/// by stat(2) calls, and so on - all files being manipulated by the operating system - are
/// identified by fids. Fids are chosen by the client. All requests on a connection share the same
/// fid space; when several clients share a connection, the agent managing the sharing must arrange
/// that no two clients choose the same fid.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct Fid(pub u32);
impl Fid {
    pub const NOFID: Fid = Fid(!0);

    pub fn read<R: io::Read>(r: &mut R) -> io::Result<Fid> {
        Ok(Fid(r.read_u32()?))
    }
    pub fn write<W: io::Write>(self, w: &mut W) -> io::Result<()> {
        w.write_u32(self.0)
    }
}
impl std::fmt::Debug for Fid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 == !0 {
            write!(f, "NOFID")
        } else {
            write!(f, "{}", self.0)
        }
    }
}
