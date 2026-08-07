//! Instrumentation the delivery pipeline exposes for the loopback
//! debugger: a synthetic feed to drive it, and the network profile that
//! shapes real transfers so a slow link can be reproduced on a desk.

#[cfg(all(
    feature = "video-debug-web",
    debug_assertions,
    not(any(target_os = "android", target_os = "ios"))
))]
pub mod feed;
pub mod network;
