//! Core utilities for serializing and deserializing the Plan 9 Remote Resource Protocol, version
//! 9p2000.
//!
//! See the `ignition-9p` crate for an implementation of the 9p2000 data types and message
//! structures built on these utilities.

mod count_prefixed_list;
mod dont_touch;
mod length_prefixed_bytes;
mod size_prefixed;
mod traits;

pub use count_prefixed_list::{BorrowedCountPrefixedList, OwnedCountPrefixedList};
pub use dont_touch::DontTouch;
pub use length_prefixed_bytes::{BorrowedLengthPrefixedBytes, OwnedLengthPrefixedBytes};
pub use size_prefixed::{BorrowedSizePrefixed, LimitedWriter, OwnedSizePrefixed};
pub use traits::{EmbeddedSize, ReadFrom, SerializedSize, WriteTo};
