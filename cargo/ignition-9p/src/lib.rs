//! An implementation of the `9P2000` wire protocol.
//!
//! Complex types can serialize and deserialize themselves in the 9P2000 wire format with methods of
//! this form:
//! ```
//! # use std::io::{self, Read, Write};
//! # trait Example {
//! fn write<W: Write>(&self, &mut W) -> io::Result<()>;
//! fn read<R: Read>(&mut R) -> io::Result<Self>;
//! # }
//! ```
//!
//! Extension traits for serializing and deserializing basic types are available in the [ext]
//! module.
//!
//! # Acknowledgement
//!
//! Documentation in this crate contains quotations and text derived from Section 5 of the Plan 9
//! manual.
//!
//! [http://man.cat-v.org/plan_9/5/](http://man.cat-v.org/plan_9/5/)

mod dont_touch;
pub mod ext;
mod fid;
mod file_type;
pub mod message;
mod open_mode;
mod qid;
mod stat;
mod stat_mode;
mod tag;
mod unix_triplet;

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
