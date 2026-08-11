//! Progressive (byte-range) delivery: the route the player hits, the
//! body stream that answers it, and the Range header it is scoped by.

pub mod capabilities;
pub mod range_header;
pub mod route;
pub(crate) mod stream;
