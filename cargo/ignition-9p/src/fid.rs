use crate::wire::{ReadWireFormat, WriteWireFormat};
use std::io::{self, Read, Write};

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
impl ReadWireFormat for Fid {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
        Ok(Fid(ReadWireFormat::read_from(r)?))
    }
}
impl WriteWireFormat for Fid {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        self.0.write_to(w)
    }
}
