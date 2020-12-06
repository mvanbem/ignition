//! Message types.

use crate::wire::{ReadFrom, WriteTo};
use crate::{Fid, OpenMode, Qid, Stat, Tag};
use ignition_9p_wire_derive::{ReadFrom, WriteTo};
use std::io::{self, Read, Write};

pub mod raw {
    use std::fmt::{self, Debug, Display, Formatter};

    use ignition_9p_wire_derive::{ReadFrom, WriteTo};

    #[derive(Clone, Copy, Eq, PartialEq, ReadFrom, WriteTo)]
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
    impl Display for MessageType {
        fn fmt(&self, f: &mut Formatter) -> fmt::Result {
            match *self {
                MessageType::TVERSION => write!(f, "TVERSION"),
                MessageType::RVERSION => write!(f, "RVERSION"),
                MessageType::TAUTH => write!(f, "TAUTH"),
                MessageType::RAUTH => write!(f, "RAUTH"),
                MessageType::TATTACH => write!(f, "TATTACH"),
                MessageType::RATTACH => write!(f, "RATTACH"),
                MessageType::RERROR => write!(f, "RERROR"),
                MessageType::TFLUSH => write!(f, "TFLUSH"),
                MessageType::RFLUSH => write!(f, "RFLUSH"),
                MessageType::TWALK => write!(f, "TWALK"),
                MessageType::RWALK => write!(f, "RWALK"),
                MessageType::TOPEN => write!(f, "TOPEN"),
                MessageType::ROPEN => write!(f, "ROPEN"),
                MessageType::TCREATE => write!(f, "TCREATE"),
                MessageType::RCREATE => write!(f, "RCREATE"),
                MessageType::TREAD => write!(f, "TREAD"),
                MessageType::RREAD => write!(f, "RREAD"),
                MessageType::TWRITE => write!(f, "TWRITE"),
                MessageType::RWRITE => write!(f, "RWRITE"),
                MessageType::TCLUNK => write!(f, "TCLUNK"),
                MessageType::RCLUNK => write!(f, "RCLUNK"),
                MessageType::TREMOVE => write!(f, "TREMOVE"),
                MessageType::RREMOVE => write!(f, "RREMOVE"),
                MessageType::TSTAT => write!(f, "TSTAT"),
                MessageType::RSTAT => write!(f, "RSTAT"),
                MessageType::TWSTAT => write!(f, "TWSTAT"),
                MessageType::RWSTAT => write!(f, "RWSTAT"),
                _ => write!(f, "{}", self.0),
            }
        }
    }
    impl Debug for MessageType {
        fn fmt(&self, f: &mut Formatter) -> fmt::Result {
            write!(f, "{}", self)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Message {
    pub tag: Tag,
    pub body: MessageBody,
}
impl ReadFrom for Message {
    fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
        let message_type = ReadFrom::read_from(r)?;
        let tag = ReadFrom::read_from(r)?;

        Ok(Message {
            tag,
            body: match message_type {
                raw::MessageType::TVERSION => MessageBody::TVersion(ReadFrom::read_from(r)?),
                raw::MessageType::RVERSION => MessageBody::RVersion(ReadFrom::read_from(r)?),
                raw::MessageType::TATTACH => MessageBody::TAttach(ReadFrom::read_from(r)?),
                raw::MessageType::RATTACH => MessageBody::RAttach(ReadFrom::read_from(r)?),
                raw::MessageType::RERROR => MessageBody::RError(ReadFrom::read_from(r)?),
                raw::MessageType::TWALK => MessageBody::TWalk(ReadFrom::read_from(r)?),
                raw::MessageType::RWALK => MessageBody::RWalk(ReadFrom::read_from(r)?),
                raw::MessageType::TOPEN => MessageBody::TOpen(ReadFrom::read_from(r)?),
                raw::MessageType::ROPEN => MessageBody::ROpen(ReadFrom::read_from(r)?),
                raw::MessageType::TCREATE => MessageBody::TCreate(ReadFrom::read_from(r)?),
                raw::MessageType::RCREATE => MessageBody::RCreate(ReadFrom::read_from(r)?),
                raw::MessageType::TREAD => MessageBody::TRead(ReadFrom::read_from(r)?),
                raw::MessageType::RREAD => MessageBody::RRead(ReadFrom::read_from(r)?),
                raw::MessageType::TWRITE => MessageBody::TWrite(ReadFrom::read_from(r)?),
                raw::MessageType::RWRITE => MessageBody::RWrite(ReadFrom::read_from(r)?),
                raw::MessageType::TCLUNK => MessageBody::TClunk(ReadFrom::read_from(r)?),
                raw::MessageType::RCLUNK => MessageBody::RClunk,
                raw::MessageType::TSTAT => MessageBody::TStat(ReadFrom::read_from(r)?),
                raw::MessageType::RSTAT => MessageBody::RStat(ReadFrom::read_from(r)?),
                raw::MessageType::TWSTAT => MessageBody::TWstat(ReadFrom::read_from(r)?),
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
impl WriteTo for Message {
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

#[derive(Clone, Debug, Eq, PartialEq, ReadFrom, WriteTo)]
pub struct TVersion {
    pub msize: u32,
    pub version: String,
}

#[derive(Clone, Debug, Eq, PartialEq, ReadFrom, WriteTo)]
pub struct RVersion {
    pub msize: u32,
    pub version: String,
}

#[derive(Clone, Debug, Eq, PartialEq, ReadFrom, WriteTo)]
pub struct TAttach {
    pub fid: Fid,
    pub afid: Fid,
    pub uname: String,
    pub aname: String,
}

#[derive(Clone, Debug, Eq, PartialEq, ReadFrom, WriteTo)]
pub struct RAttach {
    pub qid: Qid,
}

#[derive(Clone, Debug, Eq, PartialEq, ReadFrom, WriteTo)]
pub struct RError {
    pub ename: String,
}

#[derive(Clone, Debug, Eq, PartialEq, ReadFrom, WriteTo)]
pub struct TWalk {
    pub fid: Fid,
    pub newfid: Fid,
    #[ignition_9p_wire(count_prefixed = "u16")]
    pub names: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, ReadFrom, WriteTo)]
pub struct RWalk {
    #[ignition_9p_wire(count_prefixed = "u16")]
    pub qids: Vec<Qid>,
}

#[derive(Clone, Debug, Eq, PartialEq, ReadFrom, WriteTo)]
pub struct TOpen {
    pub fid: Fid,
    pub mode: OpenMode,
}

#[derive(Clone, Debug, Eq, PartialEq, ReadFrom, WriteTo)]
pub struct ROpen {
    pub qid: Qid,
    pub iounit: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, ReadFrom, WriteTo)]
pub struct TCreate {
    pub fid: Fid,
    pub name: String,
    pub perm: u32,
    pub mode: OpenMode,
}

#[derive(Clone, Debug, Eq, PartialEq, ReadFrom, WriteTo)]
pub struct RCreate {
    pub qid: Qid,
    pub iounit: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, ReadFrom, WriteTo)]
pub struct TRead {
    pub fid: Fid,
    pub offset: u64,
    pub count: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, ReadFrom, WriteTo)]
pub struct RRead {
    #[ignition_9p_wire(length_prefixed_bytes = "u32")]
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq, ReadFrom, WriteTo)]
pub struct TWrite {
    pub fid: Fid,
    pub offset: u64,
    #[ignition_9p_wire(length_prefixed_bytes = "u32")]
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq, ReadFrom, WriteTo)]
pub struct RWrite {
    pub count: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, ReadFrom, WriteTo)]
pub struct TClunk {
    pub fid: Fid,
}

#[derive(Clone, Debug, Eq, PartialEq, ReadFrom, WriteTo)]
pub struct TStat {
    pub fid: Fid,
}

#[derive(Clone, Debug, Eq, PartialEq, ReadFrom, WriteTo)]
pub struct RStat {
    #[ignition_9p_wire(size_prefixed = "u16")]
    pub stat: Stat,
}

#[derive(Clone, Debug, Eq, PartialEq, ReadFrom, WriteTo)]
pub struct TWstat {
    pub fid: Fid,
    #[ignition_9p_wire(size_prefixed = "u16")]
    pub stat: Stat,
}

#[cfg(test)]
mod tests {
    use super::{Message, MessageBody, RError, RRead, RStat, RVersion, TVersion, TWalk};
    use crate::wire::{ReadFrom, WriteTo};
    use crate::{Fid, FileType, Qid, Stat, StatMode, Tag};

    #[test]
    fn message_tversion_read() {
        let mut data: &'static [u8] = &[
            100, 0xcd, 0xab, 0x78, 0x56, 0x34, 0x12, 0x06, 0x00, 0x39, 0x50, 0x32, 0x30, 0x30, 0x30,
        ];
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
        assert_eq!(data.len(), 0);
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
            &[
                100, 0x56, 0x34, 0x78, 0x56, 0x34, 0x12, 0x06, 0x00, 0x39, 0x50, 0x32, 0x30, 0x30,
                0x30
            ],
        );
    }

    #[test]
    fn message_rversion_read() {
        let mut data: &'static [u8] = &[
            101, 0xcd, 0xab, 0x78, 0x56, 0x34, 0x12, 0x06, 0x00, 0x39, 0x50, 0x32, 0x30, 0x30, 0x30,
        ];
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
        assert_eq!(data.len(), 0);
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
            &[
                101, 0x56, 0x34, 0x78, 0x56, 0x34, 0x12, 0x06, 0x00, 0x39, 0x50, 0x32, 0x30, 0x30,
                0x30
            ],
        );
    }

    #[test]
    fn message_rerror_read() {
        let mut data: &'static [u8] = &[
            107, 0xcd, 0xab, 0x08, 0x00, 0x69, 0x74, 0x20, 0x62, 0x72, 0x6f, 0x6b, 0x65,
        ];
        assert_eq!(
            Message::read_from(&mut data).unwrap(),
            Message {
                tag: Tag(0xabcd),
                body: MessageBody::RError(RError {
                    ename: "it broke".to_string(),
                }),
            },
        );
        assert_eq!(data.len(), 0);
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
            &[107, 0xcd, 0xab, 0x08, 0x00, 0x69, 0x74, 0x20, 0x62, 0x72, 0x6f, 0x6b, 0x65],
        );
    }

    #[test]
    fn message_twalk_read() {
        let mut data: &'static [u8] = &[
            110, 0xcd, 0xab, 0x84, 0x83, 0x82, 0x81, 0x94, 0x93, 0x92, 0x91, 0x02, 0x00, 0x03,
            0x00, 'f' as u8, 'o' as u8, 'o' as u8, 0x03, 0x00, 'b' as u8, 'a' as u8, 'r' as u8,
        ];
        assert_eq!(
            Message::read_from(&mut data).unwrap(),
            Message {
                tag: Tag(0xabcd),
                body: MessageBody::TWalk(TWalk {
                    fid: Fid(0x81828384),
                    newfid: Fid(0x91929394),
                    names: vec!["foo".to_string(), "bar".to_string()],
                }),
            },
        );
        assert_eq!(data.len(), 0);
    }

    #[test]
    fn message_twalk_write() {
        let mut data = vec![];
        Message {
            tag: Tag(0xabcd),
            body: MessageBody::TWalk(TWalk {
                fid: Fid(0x81828384),
                newfid: Fid(0x91929394),
                names: vec!["foo".to_string(), "bar".to_string()],
            }),
        }
        .write_to(&mut data)
        .unwrap();
        assert_eq!(
            data.as_slice(),
            &[
                110, 0xcd, 0xab, 0x84, 0x83, 0x82, 0x81, 0x94, 0x93, 0x92, 0x91, 0x02, 0x00, 0x03,
                0x00, 'f' as u8, 'o' as u8, 'o' as u8, 0x03, 0x00, 'b' as u8, 'a' as u8, 'r' as u8
            ],
        )
    }

    #[test]
    fn message_rstat_read() {
        let mut data: &'static [u8] = &[
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
        ];
        assert_eq!(
            Message::read_from(&mut data).unwrap(),
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
            },
        );
        assert_eq!(data.len(), 0);
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
            &[
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

    #[test]
    fn message_rread_read() {
        let mut data: &'static [u8] = &[
            117, 0xcd, 0xab, 8, 0, 0, 0, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
        ];
        assert_eq!(
            Message::read_from(&mut data).unwrap(),
            Message {
                tag: Tag(0xabcd),
                body: MessageBody::RRead(RRead {
                    data: vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef],
                }),
            },
        );
        assert_eq!(data.len(), 0);
    }

    #[test]
    fn message_rread_write() {
        let mut data = vec![];
        Message {
            tag: Tag(0xabcd),
            body: MessageBody::RRead(RRead {
                data: vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef],
            }),
        }
        .write_to(&mut data)
        .unwrap();
        assert_eq!(
            data.as_slice(),
            &[117, 0xcd, 0xab, 8, 0, 0, 0, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,],
        );
    }
}
