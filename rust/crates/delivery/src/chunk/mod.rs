//! Chunk transport: one ranged HTTP GET, its body stream, and the
//! cancel handle that stops it. Everything above this folder talks in
//! chunks; nothing below it knows what a delivery plan is.

pub mod cancel;
pub mod downloader;
pub mod generation;
pub(crate) mod network;
mod response;
pub(crate) mod sink;
mod stream;
pub(crate) mod traffic;
pub(crate) mod whole_body_bound;
pub(crate) mod whole_body_limit;
pub(crate) mod whole_body_policy;
