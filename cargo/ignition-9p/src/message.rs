//! Message types.

use crate::wire::{
    BorrowedCountPrefixedList, BorrowedLengthPrefixed, OwnedCountPrefixedList, OwnedLengthPrefixed,
    ReadWireFormat, WriteWireFormat,
};
use crate::{Fid, OpenMode, Qid, Stat, Tag};
use std::io::{self, Read, Write};

pub mod raw {
    use crate::wire::{ReadWireFormat, WriteWireFormat};
    use std::io::{self, Read, Write};

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct MessageType(pub u8);
    impl MessageType {
        pub const TVERSION: MessageType = MessageType(100);
        pub const RVERSION: MessageType = MessageType(101);
        pub const TAUTH: MessageType = MessageType(102);
        pub const RAUTH: MessageType = MessageType(103);
        pub const TATTACH: MessageType = MessageType(104);
        pub const RATTACH: MessageType = MessageType(105);
        // There is no TERROR.
        pub const RERROR: MessageType = MessageType(107);
        pub const TFLUSH: MessageType = MessageType(108);
        pub const RFLUSH: MessageType = MessageType(109);
        pub const TWALK: MessageType = MessageType(110);
        pub const RWALK: MessageType = MessageType(111);
        pub const TOPEN: MessageType = MessageType(112);
        pub const ROPEN: MessageType = MessageType(113);
        pub const TCREATE: MessageType = MessageType(114);
        pub const RCREATE: MessageType = MessageType(115);
        pub const TREAD: MessageType = MessageType(116);
        pub const RREAD: MessageType = MessageType(117);
        pub const TWRITE: MessageType = MessageType(118);
        pub const RWRITE: MessageType = MessageType(119);
        pub const TCLUNK: MessageType = MessageType(120);
        pub const RCLUNK: MessageType = MessageType(121);
        pub const TREMOVE: MessageType = MessageType(122);
        pub const RREMOVE: MessageType = MessageType(123);
        pub const TSTAT: MessageType = MessageType(124);
        pub const RSTAT: MessageType = MessageType(125);
        pub const TWSTAT: MessageType = MessageType(126);
        pub const RWSTAT: MessageType = MessageType(127);
    }
    impl ReadWireFormat for MessageType {
        fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
            Ok(MessageType(ReadWireFormat::read_from(r)?))
        }
    }
    impl WriteWireFormat for MessageType {
        fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
            self.0.write_to(w)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Message {
    pub tag: Tag,
    pub body: MessageBody,
}
impl ReadWireFormat for Message {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
        let message_type = ReadWireFormat::read_from(r)?;
        let tag = ReadWireFormat::read_from(r)?;

        Ok(Message {
            tag,
            body: match message_type {
                raw::MessageType::TVERSION => MessageBody::TVersion(ReadWireFormat::read_from(r)?),
                raw::MessageType::RVERSION => MessageBody::RVersion(ReadWireFormat::read_from(r)?),
                raw::MessageType::TATTACH => MessageBody::TAttach(ReadWireFormat::read_from(r)?),
                raw::MessageType::RATTACH => MessageBody::RAttach(ReadWireFormat::read_from(r)?),
                raw::MessageType::RERROR => MessageBody::RError(ReadWireFormat::read_from(r)?),
                raw::MessageType::TWALK => MessageBody::TWalk(ReadWireFormat::read_from(r)?),
                raw::MessageType::RWALK => MessageBody::RWalk(ReadWireFormat::read_from(r)?),
                raw::MessageType::TOPEN => MessageBody::TOpen(ReadWireFormat::read_from(r)?),
                raw::MessageType::ROPEN => MessageBody::ROpen(ReadWireFormat::read_from(r)?),
                raw::MessageType::TCREATE => MessageBody::TCreate(ReadWireFormat::read_from(r)?),
                raw::MessageType::RCREATE => MessageBody::RCreate(ReadWireFormat::read_from(r)?),
                raw::MessageType::TREAD => MessageBody::TRead(ReadWireFormat::read_from(r)?),
                raw::MessageType::RREAD => MessageBody::RRead(ReadWireFormat::read_from(r)?),
                raw::MessageType::TWRITE => MessageBody::TWrite(ReadWireFormat::read_from(r)?),
                raw::MessageType::RWRITE => MessageBody::RWrite(ReadWireFormat::read_from(r)?),
                raw::MessageType::TCLUNK => MessageBody::TClunk(ReadWireFormat::read_from(r)?),
                raw::MessageType::RCLUNK => MessageBody::RClunk,
                raw::MessageType::TSTAT => MessageBody::TStat(ReadWireFormat::read_from(r)?),
                raw::MessageType::RSTAT => MessageBody::RStat(ReadWireFormat::read_from(r)?),
                raw::MessageType::TWSTAT => MessageBody::TWstat(ReadWireFormat::read_from(r)?),
                raw::MessageType::RWSTAT => MessageBody::RWstat,
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        ReadMessageError::UnsupportedMessageType { message_type },
                    ))
                }
            },
        })
    }
}
impl WriteWireFormat for Message {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        self.body.message_type().write_to(w)?;
        self.tag.write_to(w)?;
        match &self.body {
            MessageBody::TVersion(v) => v.write_to(w),
            MessageBody::RVersion(v) => v.write_to(w),
            MessageBody::TAttach(v) => v.write_to(w),
            MessageBody::RAttach(v) => v.write_to(w),
            MessageBody::RError(v) => v.write_to(w),
            MessageBody::TWalk(v) => v.write_to(w),
            MessageBody::RWalk(v) => v.write_to(w),
            MessageBody::TOpen(v) => v.write_to(w),
            MessageBody::ROpen(v) => v.write_to(w),
            MessageBody::TCreate(v) => v.write_to(w),
            MessageBody::RCreate(v) => v.write_to(w),
            MessageBody::TRead(v) => v.write_to(w),
            MessageBody::RRead(v) => v.write_to(w),
            MessageBody::TWrite(v) => v.write_to(w),
            MessageBody::RWrite(v) => v.write_to(w),
            MessageBody::TClunk(v) => v.write_to(w),
            MessageBody::RClunk => Ok(()),
            MessageBody::TStat(v) => v.write_to(w),
            MessageBody::RStat(v) => v.write_to(w),
            MessageBody::TWstat(v) => v.write_to(w),
            MessageBody::RWstat => Ok(()),
        }
    }
}

#[derive(Debug)]
pub enum ReadMessageError {
    UnsupportedMessageType { message_type: raw::MessageType },
}
impl std::fmt::Display for ReadMessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadMessageError::UnsupportedMessageType { message_type } => {
                write!(f, "unsupported message type: {}", message_type.0)
            }
        }
    }
}
impl std::error::Error for ReadMessageError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MessageBody {
    TVersion(TVersion),
    RVersion(RVersion),
    TAttach(TAttach),
    RAttach(RAttach),
    RError(RError),
    TWalk(TWalk),
    RWalk(RWalk),
    TOpen(TOpen),
    ROpen(ROpen),
    TCreate(TCreate),
    RCreate(RCreate),
    TRead(TRead),
    RRead(RRead),
    TWrite(TWrite),
    RWrite(RWrite),
    TClunk(TClunk),
    RClunk,
    TStat(TStat),
    RStat(RStat),
    TWstat(TWstat),
    RWstat,
}
impl MessageBody {
    pub fn message_type(&self) -> raw::MessageType {
        match self {
            MessageBody::TVersion(_) => raw::MessageType::TVERSION,
            MessageBody::RVersion(_) => raw::MessageType::RVERSION,
            MessageBody::TAttach(_) => raw::MessageType::TATTACH,
            MessageBody::RAttach(_) => raw::MessageType::RATTACH,
            MessageBody::RError(_) => raw::MessageType::RERROR,
            MessageBody::TWalk(_) => raw::MessageType::TWALK,
            MessageBody::RWalk(_) => raw::MessageType::RWALK,
            MessageBody::TOpen(_) => raw::MessageType::TOPEN,
            MessageBody::ROpen(_) => raw::MessageType::ROPEN,
            MessageBody::TCreate(_) => raw::MessageType::TCREATE,
            MessageBody::RCreate(_) => raw::MessageType::RCREATE,
            MessageBody::TRead(_) => raw::MessageType::TREAD,
            MessageBody::RRead(_) => raw::MessageType::RREAD,
            MessageBody::TWrite(_) => raw::MessageType::TWRITE,
            MessageBody::RWrite(_) => raw::MessageType::RWRITE,
            MessageBody::TClunk(_) => raw::MessageType::TCLUNK,
            MessageBody::RClunk => raw::MessageType::RCLUNK,
            MessageBody::TStat(_) => raw::MessageType::TSTAT,
            MessageBody::RStat(_) => raw::MessageType::RSTAT,
            MessageBody::TWstat(_) => raw::MessageType::TWSTAT,
            MessageBody::RWstat => raw::MessageType::RWSTAT,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TVersion {
    pub msize: u32,
    pub version: String,
}
impl ReadWireFormat for TVersion {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
        let msize = ReadWireFormat::read_from(r)?;
        let version = ReadWireFormat::read_from(r)?;
        Ok(TVersion { msize, version })
    }
}
impl WriteWireFormat for TVersion {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        self.msize.write_to(w)?;
        self.version.write_to(w)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RVersion {
    pub msize: u32,
    pub version: String,
}
impl ReadWireFormat for RVersion {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
        let msize = ReadWireFormat::read_from(r)?;
        let version = ReadWireFormat::read_from(r)?;
        Ok(RVersion { msize, version })
    }
}
impl WriteWireFormat for RVersion {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        self.msize.write_to(w)?;
        self.version.write_to(w)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TAttach {
    pub fid: Fid,
    pub afid: Fid,
    pub uname: String,
    pub aname: String,
}
impl ReadWireFormat for TAttach {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
        let fid = ReadWireFormat::read_from(r)?;
        let afid = ReadWireFormat::read_from(r)?;
        let uname = ReadWireFormat::read_from(r)?;
        let aname = ReadWireFormat::read_from(r)?;
        Ok(TAttach {
            fid,
            afid,
            uname,
            aname,
        })
    }
}
impl WriteWireFormat for TAttach {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        self.fid.write_to(w)?;
        self.afid.write_to(w)?;
        self.uname.write_to(w)?;
        self.aname.write_to(w)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RAttach {
    pub qid: Qid,
}
impl ReadWireFormat for RAttach {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
        Ok(RAttach {
            qid: ReadWireFormat::read_from(r)?,
        })
    }
}
impl WriteWireFormat for RAttach {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        self.qid.write_to(w)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RError {
    pub ename: String,
}
impl ReadWireFormat for RError {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
        Ok(RError {
            ename: ReadWireFormat::read_from(r)?,
        })
    }
}
impl WriteWireFormat for RError {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        self.ename.write_to(w)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TWalk {
    pub fid: Fid,
    pub newfid: Fid,
    pub names: Vec<String>,
}
impl ReadWireFormat for TWalk {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
        let fid = ReadWireFormat::read_from(r)?;
        let newfid = ReadWireFormat::read_from(r)?;
        let names = OwnedCountPrefixedList::<u16, _>::read_from(r)?.into_inner();
        Ok(TWalk { fid, newfid, names })
    }
}
impl WriteWireFormat for TWalk {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        self.fid.write_to(w)?;
        self.newfid.write_to(w)?;
        BorrowedCountPrefixedList::<u16, _>::new(&self.names).write_to(w)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RWalk {
    pub qids: Vec<Qid>,
}
impl ReadWireFormat for RWalk {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
        Ok(RWalk {
            qids: OwnedCountPrefixedList::<u16, _>::read_from(r)?.into_inner(),
        })
    }
}
impl WriteWireFormat for RWalk {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        BorrowedCountPrefixedList::<u16, _>::new(&self.qids).write_to(w)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TOpen {
    pub fid: Fid,
    pub mode: OpenMode,
}
impl ReadWireFormat for TOpen {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
        let fid = ReadWireFormat::read_from(r)?;
        let mode = ReadWireFormat::read_from(r)?;
        Ok(TOpen { fid, mode })
    }
}
impl WriteWireFormat for TOpen {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        self.fid.write_to(w)?;
        self.mode.write_to(w)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ROpen {
    pub qid: Qid,
    pub iounit: u32,
}
impl ReadWireFormat for ROpen {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
        let qid = ReadWireFormat::read_from(r)?;
        let iounit = ReadWireFormat::read_from(r)?;
        Ok(ROpen { qid, iounit })
    }
}
impl WriteWireFormat for ROpen {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        self.qid.write_to(w)?;
        self.iounit.write_to(w)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TCreate {
    pub fid: Fid,
    pub name: String,
    pub perm: u32,
    pub mode: OpenMode,
}
impl ReadWireFormat for TCreate {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
        let fid = ReadWireFormat::read_from(r)?;
        let name = ReadWireFormat::read_from(r)?;
        let perm = ReadWireFormat::read_from(r)?;
        let mode = ReadWireFormat::read_from(r)?;
        Ok(TCreate {
            fid,
            name,
            perm,
            mode,
        })
    }
}
impl WriteWireFormat for TCreate {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        self.fid.write_to(w)?;
        self.name.write_to(w)?;
        self.perm.write_to(w)?;
        self.mode.write_to(w)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RCreate {
    pub qid: Qid,
    pub iounit: u32,
}
impl ReadWireFormat for RCreate {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
        let qid = ReadWireFormat::read_from(r)?;
        let iounit = ReadWireFormat::read_from(r)?;
        Ok(RCreate { qid, iounit })
    }
}
impl WriteWireFormat for RCreate {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        self.qid.write_to(w)?;
        self.iounit.write_to(w)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TRead {
    pub fid: Fid,
    pub offset: u64,
    pub count: u32,
}
impl ReadWireFormat for TRead {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
        let fid = ReadWireFormat::read_from(r)?;
        let offset = ReadWireFormat::read_from(r)?;
        let count = ReadWireFormat::read_from(r)?;
        Ok(TRead { fid, offset, count })
    }
}
impl WriteWireFormat for TRead {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        self.fid.write_to(w)?;
        self.offset.write_to(w)?;
        self.count.write_to(w)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RRead {
    pub data: Vec<u8>,
}
impl ReadWireFormat for RRead {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
        Ok(RRead {
            data: OwnedCountPrefixedList::<u32, _>::read_from(r)?.into_inner(),
        })
    }
}
impl WriteWireFormat for RRead {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        BorrowedCountPrefixedList::<u32, _>::new(&self.data).write_to(w)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TWrite {
    pub fid: Fid,
    pub offset: u64,
    pub data: Vec<u8>,
}
impl ReadWireFormat for TWrite {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
        let fid = ReadWireFormat::read_from(r)?;
        let offset = ReadWireFormat::read_from(r)?;
        let data = OwnedCountPrefixedList::<u32, _>::read_from(r)?.into_inner();
        Ok(TWrite { fid, offset, data })
    }
}
impl WriteWireFormat for TWrite {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        self.fid.write_to(w)?;
        self.offset.write_to(w)?;
        BorrowedCountPrefixedList::<u32, _>::new(&self.data).write_to(w)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RWrite {
    pub count: u32,
}
impl ReadWireFormat for RWrite {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
        Ok(RWrite {
            count: ReadWireFormat::read_from(r)?,
        })
    }
}
impl WriteWireFormat for RWrite {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        self.count.write_to(w)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TClunk {
    pub fid: Fid,
}
impl ReadWireFormat for TClunk {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
        Ok(TClunk {
            fid: ReadWireFormat::read_from(r)?,
        })
    }
}
impl WriteWireFormat for TClunk {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        self.fid.write_to(w)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TStat {
    pub fid: Fid,
}
impl ReadWireFormat for TStat {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
        Ok(TStat {
            fid: ReadWireFormat::read_from(r)?,
        })
    }
}
impl WriteWireFormat for TStat {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        self.fid.write_to(w)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RStat {
    pub stat: Stat,
}
impl ReadWireFormat for RStat {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
        Ok(RStat {
            stat: OwnedLengthPrefixed::<u16, _>::read_from(r)?.into_inner(),
        })
    }
}
impl WriteWireFormat for RStat {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        BorrowedLengthPrefixed::<u16, _>::new(&self.stat).write_to(w)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TWstat {
    pub fid: Fid,
    pub stat: Stat,
}
impl ReadWireFormat for TWstat {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
        let fid = ReadWireFormat::read_from(r)?;
        let stat = ReadWireFormat::read_from(r)?;
        Ok(TWstat { fid, stat })
    }
}
impl WriteWireFormat for TWstat {
    fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        self.fid.write_to(w)?;
        self.stat.write_to(w)
    }
}

#[cfg(test)]
mod tests {
    use super::{Message, MessageBody, RError, RStat, RVersion, TVersion};
    use crate::wire::{ReadWireFormat, WriteWireFormat};
    use crate::{FileType, Qid, Stat, StatMode, Tag};
    use std::io;

    #[test]
    fn message_tversion_read() {
        let mut data = io::Cursor::new([
            100, 0xcd, 0xab, 0x78, 0x56, 0x34, 0x12, 0x06, 0x00, 0x39, 0x50, 0x32, 0x30, 0x30, 0x30,
        ]);
        assert_eq!(
            Message::read_from(&mut data).unwrap(),
            Message {
                tag: Tag(0xabcd),
                body: MessageBody::TVersion(TVersion {
                    msize: 0x12345678,
                    version: "9P2000".to_string(),
                }),
            },
        );
    }

    #[test]
    fn message_tversion_write() {
        let mut data = vec![];
        Message {
            tag: Tag(0x3456),
            body: MessageBody::TVersion(TVersion {
                msize: 0x12345678,
                version: "9P2000".to_string(),
            }),
        }
        .write_to(&mut data)
        .unwrap();
        assert_eq!(
            data.as_slice(),
            [
                100, 0x56, 0x34, 0x78, 0x56, 0x34, 0x12, 0x06, 0x00, 0x39, 0x50, 0x32, 0x30, 0x30,
                0x30
            ],
        );
    }

    #[test]
    fn message_rversion_read() {
        let mut data = io::Cursor::new([
            101, 0xcd, 0xab, 0x78, 0x56, 0x34, 0x12, 0x06, 0x00, 0x39, 0x50, 0x32, 0x30, 0x30, 0x30,
        ]);
        assert_eq!(
            Message::read_from(&mut data).unwrap(),
            Message {
                tag: Tag(0xabcd),
                body: MessageBody::RVersion(RVersion {
                    msize: 0x12345678,
                    version: "9P2000".to_string(),
                }),
            },
        );
    }

    #[test]
    fn message_rversion_write() {
        let mut data = vec![];
        Message {
            tag: Tag(0x3456),
            body: MessageBody::RVersion(RVersion {
                msize: 0x12345678,
                version: "9P2000".to_string(),
            }),
        }
        .write_to(&mut data)
        .unwrap();
        assert_eq!(
            data.as_slice(),
            [
                101, 0x56, 0x34, 0x78, 0x56, 0x34, 0x12, 0x06, 0x00, 0x39, 0x50, 0x32, 0x30, 0x30,
                0x30
            ],
        );
    }

    #[test]
    fn message_rerror_read() {
        let mut data = io::Cursor::new([
            107, 0xcd, 0xab, 0x08, 0x00, 0x69, 0x74, 0x20, 0x62, 0x72, 0x6f, 0x6b, 0x65,
        ]);
        assert_eq!(
            Message::read_from(&mut data).unwrap(),
            Message {
                tag: Tag(0xabcd),
                body: MessageBody::RError(RError {
                    ename: "it broke".to_string(),
                }),
            },
        );
    }

    #[test]
    fn message_rerror_write() {
        let mut data = vec![];
        Message {
            tag: Tag(0xabcd),
            body: MessageBody::RError(RError {
                ename: "it broke".to_string(),
            }),
        }
        .write_to(&mut data)
        .unwrap();
        assert_eq!(
            data.as_slice(),
            [107, 0xcd, 0xab, 0x08, 0x00, 0x69, 0x74, 0x20, 0x62, 0x72, 0x6f, 0x6b, 0x65],
        );
    }

    #[test]
    fn message_rstat_read() {
        let mut data = io::Cursor::new([
            107, 0xcd, 0xab, 0x08, 0x00, 0x69, 0x74, 0x20, 0x62, 0x72, 0x6f, 0x6b, 0x65,
        ]);
        assert_eq!(
            Message::read_from(&mut data).unwrap(),
            Message {
                tag: Tag(0xabcd),
                body: MessageBody::RError(RError {
                    ename: "it broke".to_string(),
                }),
            },
        );
    }

    #[test]
    fn message_rstat_write() {
        let mut data = vec![];
        Message {
            tag: Tag(0xabcd),
            body: MessageBody::RStat(RStat {
                stat: Stat {
                    kernel_type: 0,
                    kernel_dev: 0,
                    qid: Qid {
                        file_type: FileType::default(),
                        version: 0,
                        path: 0,
                    },
                    mode: StatMode::default(),
                    atime: 0,
                    mtime: 0,
                    length: 0,
                    name: "/".to_string(),
                    uid: "".to_string(),
                    gid: "".to_string(),
                    muid: "".to_string(),
                },
            }),
        }
        .write_to(&mut data)
        .unwrap();
        assert_eq!(
            data.as_slice(),
            [
                125, // message_type
                0xcd, 0xab, // tag
                50, 0, // stat outer length
                48, 0, // stat inner length
                0x00, 0x00, // kernel_type
                0x00, 0x00, 0x00, 0x00, // kernel_dev
                0x00, // qid.file_type
                0x00, 0x00, 0x00, 0x00, // qid.version
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // qid.path
                0x00, 0x00, 0x00, 0x00, // mode
                0x00, 0x00, 0x00, 0x00, // atime
                0x00, 0x00, 0x00, 0x00, // mtime
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // length
                0x01, 0x00, '/' as u8, // name
                0x00, 0x00, // uid
                0x00, 0x00, // gid
                0x00, 0x00, // muid
            ],
        );
    }
}
