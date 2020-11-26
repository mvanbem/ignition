//! Message types.

use crate::ext::{ReadBytesExt, WriteBytesExt};
use crate::{Fid, OpenMode, Qid, Stat, Tag};
use std::io;

pub mod raw {
    use crate::ext::{ReadBytesExt, WriteBytesExt};
    use crate::Tag;
    use std::io;

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

        pub fn read<R: io::Read>(r: &mut R) -> io::Result<MessageType> {
            Ok(MessageType(r.read_u8()?))
        }

        pub fn write<W: io::Write>(self, w: &mut W) -> io::Result<()> {
            w.write_u8(self.0)
        }
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct Header {
        pub message_type: MessageType,
        pub tag: Tag,
    }
    impl Header {
        pub fn read<R: io::Read>(r: &mut R) -> io::Result<Header> {
            let message_type = MessageType::read(r)?;
            let tag = Tag::read(r)?;
            Ok(Header { message_type, tag })
        }

        pub fn write<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
            self.message_type.write(w)?;
            self.tag.write(w)
        }
    }
    #[cfg(test)]
    mod tests {
        use super::{Header, MessageType};
        use crate::Tag;
        use std::io;

        #[test]
        fn header_read() {
            let mut data = io::Cursor::new([100, 0xcd, 0xab]);
            assert_eq!(
                Header::read(&mut data).unwrap(),
                Header {
                    message_type: MessageType::TVERSION,
                    tag: Tag(0xabcd),
                },
            );
        }

        #[test]
        fn header_write() {
            let mut data = vec![];
            Header {
                message_type: MessageType::RVERSION,
                tag: Tag(0x1234),
            }
            .write(&mut data)
            .unwrap();
            assert_eq!(data, &[101, 0x34, 0x12]);
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Message {
    pub tag: Tag,
    pub body: MessageBody,
}
impl Message {
    pub fn read<R: io::Read>(r: &mut R) -> io::Result<Message> {
        let header = raw::Header::read(r)?;
        Ok(Message {
            tag: header.tag,
            body: match header.message_type {
                raw::MessageType::TVERSION => {
                    let msize = r.read_u32()?;
                    let version = r.read_string()?;
                    MessageBody::TVersion(TVersion { msize, version })
                }
                raw::MessageType::RVERSION => {
                    let msize = r.read_u32()?;
                    let version = r.read_string()?;
                    MessageBody::RVersion(RVersion { msize, version })
                }
                raw::MessageType::TATTACH => {
                    let fid = Fid::read(r)?;
                    let afid = Fid::read(r)?;
                    let uname = r.read_string()?;
                    let aname = r.read_string()?;
                    MessageBody::TAttach(TAttach {
                        fid,
                        afid,
                        uname,
                        aname,
                    })
                }
                raw::MessageType::RATTACH => MessageBody::RAttach(RAttach { qid: Qid::read(r)? }),
                raw::MessageType::RERROR => MessageBody::RError(RError {
                    ename: r.read_string()?,
                }),
                raw::MessageType::TWALK => {
                    let fid = Fid::read(r)?;
                    let newfid = Fid::read(r)?;
                    let names = r.read_array(|r| r.read_string())?;
                    MessageBody::TWalk(TWalk { fid, newfid, names })
                }
                raw::MessageType::RWALK => MessageBody::RWalk(RWalk {
                    qids: r.read_array(|r| Qid::read(r))?,
                }),
                raw::MessageType::TOPEN => {
                    let fid = Fid::read(r)?;
                    let mode = OpenMode(r.read_u8()?);
                    MessageBody::TOpen(TOpen { fid, mode })
                }
                raw::MessageType::ROPEN => {
                    let qid = Qid::read(r)?;
                    let iounit = r.read_u32()?;
                    MessageBody::ROpen(ROpen { qid, iounit })
                }
                raw::MessageType::TCREATE => {
                    let fid = Fid::read(r)?;
                    let name = r.read_string()?;
                    let perm = r.read_u32()?;
                    let mode = OpenMode(r.read_u8()?);
                    MessageBody::TCreate(TCreate {
                        fid,
                        name,
                        perm,
                        mode,
                    })
                }
                raw::MessageType::RCREATE => {
                    let qid = Qid::read(r)?;
                    let iounit = r.read_u32()?;
                    MessageBody::RCreate(RCreate { qid, iounit })
                }
                raw::MessageType::TREAD => {
                    let fid = Fid::read(r)?;
                    let offset = r.read_u64()?;
                    let count = r.read_u32()?;
                    MessageBody::TRead(TRead { fid, offset, count })
                }
                raw::MessageType::RREAD => {
                    let data = r.read_u32_prefixed_bytes()?;
                    MessageBody::RRead(RRead { data })
                }
                raw::MessageType::TWRITE => {
                    let fid = Fid::read(r)?;
                    let offset = r.read_u64()?;
                    let data = r.read_u32_prefixed_bytes()?;
                    MessageBody::TWrite(TWrite { fid, offset, data })
                }
                raw::MessageType::RWRITE => {
                    let count = r.read_u32()?;
                    MessageBody::RWrite(RWrite { count })
                }
                raw::MessageType::TCLUNK => {
                    let fid = Fid::read(r)?;
                    MessageBody::TClunk(TClunk { fid })
                }
                raw::MessageType::RCLUNK => MessageBody::RClunk,
                raw::MessageType::TSTAT => {
                    let fid = Fid::read(r)?;
                    MessageBody::TStat(TStat { fid })
                }
                raw::MessageType::RSTAT => {
                    let stat = Stat::read(r)?;
                    MessageBody::RStat(RStat { stat })
                }
                raw::MessageType::TWSTAT => {
                    let fid = Fid::read(r)?;
                    let stat = Stat::read(r)?;
                    MessageBody::TWstat(TWstat { fid, stat })
                }
                raw::MessageType::RWSTAT => MessageBody::RWstat,
                message_type => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        ReadMessageError::UnsupportedMessageType { message_type },
                    ))
                }
            },
        })
    }

    pub fn write<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        raw::Header {
            message_type: self.body.message_type(),
            tag: self.tag,
        }
        .write(w)?;
        match self.body {
            MessageBody::TVersion(TVersion { msize, ref version }) => {
                w.write_u32(msize)?;
                w.write_string(version)
            }
            MessageBody::RVersion(RVersion { msize, ref version }) => {
                w.write_u32(msize)?;
                w.write_string(version)
            }
            MessageBody::TAttach(TAttach {
                fid,
                afid,
                ref uname,
                ref aname,
            }) => {
                fid.write(w)?;
                afid.write(w)?;
                w.write_string(uname)?;
                w.write_string(aname)
            }
            MessageBody::RAttach(RAttach { qid }) => qid.write(w),
            MessageBody::RError(RError { ref ename }) => w.write_string(ename),
            MessageBody::TWalk(TWalk {
                fid,
                newfid,
                ref names,
            }) => {
                fid.write(w)?;
                newfid.write(w)?;
                w.write_array(names, |w, name| w.write_string(name))
            }
            MessageBody::RWalk(RWalk { ref qids }) => w.write_array(qids, |w, qid| qid.write(w)),
            MessageBody::TOpen(TOpen { fid, mode }) => {
                fid.write(w)?;
                w.write_u8(mode.into())
            }
            MessageBody::ROpen(ROpen { qid, iounit }) => {
                qid.write(w)?;
                w.write_u32(iounit)
            }
            MessageBody::TCreate(TCreate {
                fid,
                ref name,
                perm,
                mode,
            }) => {
                fid.write(w)?;
                w.write_string(name)?;
                w.write_u32(perm)?;
                w.write_u8(mode.into())
            }
            MessageBody::RCreate(RCreate { qid, iounit }) => {
                qid.write(w)?;
                w.write_u32(iounit)
            }
            MessageBody::TRead(TRead { fid, offset, count }) => {
                fid.write(w)?;
                w.write_u64(offset)?;
                w.write_u32(count)
            }
            MessageBody::RRead(RRead { ref data }) => w.write_u32_prefixed_bytes(data),
            MessageBody::TWrite(TWrite {
                fid,
                offset,
                ref data,
            }) => {
                fid.write(w)?;
                w.write_u64(offset)?;
                w.write_u32_prefixed_bytes(data)
            }
            MessageBody::RWrite(RWrite { count }) => w.write_u32(count),
            MessageBody::TClunk(TClunk { fid }) => fid.write(w),
            MessageBody::RClunk => Ok(()),
            MessageBody::TStat(TStat { fid }) => fid.write(w),
            MessageBody::RStat(RStat { ref stat }) => stat.write(w),
            MessageBody::TWstat(TWstat { fid, ref stat }) => {
                fid.write(w)?;
                stat.write(w)
            }
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RVersion {
    pub msize: u32,
    pub version: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TAttach {
    pub fid: Fid,
    pub afid: Fid,
    pub uname: String,
    pub aname: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RAttach {
    pub qid: Qid,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RError {
    pub ename: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TWalk {
    pub fid: Fid,
    pub newfid: Fid,
    pub names: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RWalk {
    pub qids: Vec<Qid>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TOpen {
    pub fid: Fid,
    pub mode: OpenMode,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ROpen {
    pub qid: Qid,
    pub iounit: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TCreate {
    pub fid: Fid,
    pub name: String,
    pub perm: u32,
    pub mode: OpenMode,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RCreate {
    pub qid: Qid,
    pub iounit: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TRead {
    pub fid: Fid,
    pub offset: u64,
    pub count: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RRead {
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TWrite {
    pub fid: Fid,
    pub offset: u64,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RWrite {
    pub count: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TClunk {
    pub fid: Fid,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TStat {
    pub fid: Fid,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RStat {
    pub stat: Stat,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TWstat {
    pub fid: Fid,
    pub stat: Stat,
}

#[cfg(test)]
mod tests {
    use super::{Message, MessageBody, RError, RVersion, TVersion};
    use crate::Tag;
    use std::io;

    #[test]
    fn message_tversion_read() {
        let mut data = io::Cursor::new([
            100, 0xcd, 0xab, 0x78, 0x56, 0x34, 0x12, 0x06, 0x00, 0x39, 0x50, 0x32, 0x30, 0x30, 0x30,
        ]);
        assert_eq!(
            Message::read(&mut data).unwrap(),
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
        .write(&mut data)
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
            Message::read(&mut data).unwrap(),
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
        .write(&mut data)
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
            Message::read(&mut data).unwrap(),
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
        .write(&mut data)
        .unwrap();
        assert_eq!(
            data.as_slice(),
            [107, 0xcd, 0xab, 0x08, 0x00, 0x69, 0x74, 0x20, 0x62, 0x72, 0x6f, 0x6b, 0x65],
        );
    }
}
