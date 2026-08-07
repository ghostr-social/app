//! The loopback debug web app. Desktop debug builds only — the whole
//! folder is gated once, at its declaration in `lib.rs`, so nothing
//! inside repeats the target and feature checks.

pub mod assets;
pub mod hls;
pub mod http;
pub mod state;
pub mod videos;
