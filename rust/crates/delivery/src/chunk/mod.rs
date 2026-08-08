//! Chunk transport: one ranged HTTP GET, its body stream, and the
//! cancel handle that stops it. Everything above this folder talks in
//! chunks; nothing below it knows what a delivery plan is.

pub mod cancel;
pub mod downloader;
pub(crate) mod network;
mod response;
mod stream;
