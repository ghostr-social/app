//! Rust-owned HLS playback: the session registry the router serves
//! from, the resource budget that admits a session, and the routes.

mod cached;
pub(crate) mod capability;
pub mod playback;
pub(crate) mod routes;
pub mod sessions;
pub mod state;
mod transfer;
pub mod types;
