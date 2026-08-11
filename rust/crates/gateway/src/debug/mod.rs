//! The loopback debug web app. Desktop debug builds only — the whole
//! folder is gated once, at its declaration in `lib.rs`, so nothing
//! inside repeats the target and feature checks.

pub(crate) mod assets;
pub(crate) mod hls;
pub(crate) mod http;
pub mod media;
pub mod state;
pub mod videos;
