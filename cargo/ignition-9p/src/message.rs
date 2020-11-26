use crate::mode::Mode;
use crate::{Fid, Qid, Stat, Tag};

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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Message {
    pub tag: Tag,
    pub body: MessageBody,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MessageBody {
    TVersion {
        msize: u32,
        version: String,
    },
    RVersion {
        msize: u32,
        version: String,
    },
    TAttach {
        fid: Fid,
        afid: Fid,
        uname: String,
        aname: String,
    },
    RAttach {
        qid: Qid,
    },
    RError {
        ename: String,
    },
    TWalk {
        fid: Fid,
        newfid: Fid,
        names: Vec<String>,
    },
    RWalk {
        qids: Vec<Qid>,
    },
    TOpen {
        fid: Fid,
        mode: Mode,
    },
    ROpen {
        qid: Qid,
        iounit: u32,
    },
    TCreate {
        fid: Fid,
        name: String,
        perm: u32,
        mode: Mode,
    },
    RCreate {
        qid: Qid,
        iounit: u32,
    },
    TRead {
        fid: Fid,
        offset: u64,
        count: u32,
    },
    RRead {
        data: Vec<u8>,
    },
    TWrite {
        fid: Fid,
        offset: u64,
        data: Vec<u8>,
    },
    RWrite {
        count: u32,
    },
    TClunk {
        fid: Fid,
    },
    RClunk {},
    TStat {
        fid: Fid,
    },
    RStat {
        stat: Stat,
    },
    TWStat {
        fid: Fid,
        stat: Stat,
    },
    RWStat {},
}
