//! An implementation of the `9P2000` wire protocol.
//!
//! Serialization and deserialization are supported through the
//! [`WriteWireFormat`](wire::WriteWireFormat) and [`ReadWireFormat`](wire::ReadWireFormat) traits
//! in the [`wire`] module.
//!
//! # Example
//!
//! ```
//! use ignition_9p::Tag;
//! use ignition_9p::message::{Message, MessageBody, TVersion};
//! use ignition_9p::wire::ReadWireFormat;
//!
//! let mut data: &'static [u8] = &[
//!     100, 0xcd, 0xab, 0x78, 0x56, 0x34, 0x12, 0x06, 0x00, 0x39, 0x50, 0x32, 0x30, 0x30, 0x30,
//! ];
//! let msg = Message::read_from(&mut data).unwrap();
//! assert_eq!(
//!     msg,
//!     Message {
//!         tag: Tag(0xabcd),
//!         body: MessageBody::TVersion(TVersion {
//!             msize: 0x12345678,
//!             version: "9P2000".to_string(),
//!         }),
//!     },
//! );
//! ```
//!
//! # Acknowledgement
//!
//! Documentation in this crate contains quotations and text derived from Section 5 of the Plan 9
//! manual.
//!
//! [http://man.cat-v.org/plan_9/5/](http://man.cat-v.org/plan_9/5/)

mod dont_touch;
mod fid;
mod file_type;
pub mod message;
mod open_mode;
mod qid;
mod stat;
mod stat_mode;
mod tag;
mod unix_triplet;
pub mod wire;

pub use crate::{
    dont_touch::DontTouch,
    fid::Fid,
    file_type::FileType,
    open_mode::{OpenAccess, OpenMode},
    qid::Qid,
    stat::Stat,
    stat_mode::StatMode,
    tag::Tag,
    unix_triplet::UnixTriplet,
};
