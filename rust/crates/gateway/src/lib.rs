//! `ghostr-gateway` — extracted from `rust_lib_ghostr::video`.
//!
//! The loopback HTTP server the player talks to. `router`, `runtime`,
//! and `delivery` compose the crate; each folder below owns one way of
//! serving media.

pub(crate) mod delivery;
pub mod router;
pub mod runtime;

pub mod hls;
pub mod progressive;

#[cfg(all(
    feature = "video-debug-web",
    debug_assertions,
    not(any(target_os = "android", target_os = "ios"))
))]
pub mod debug;
