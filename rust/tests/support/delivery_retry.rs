//! Shared fixtures for the per-source retry policy tests.

use rust_lib_ghostr::engine::PostId;
use rust_lib_ghostr::video::delivery_retry::{RetryPolicy, Source};
use std::time::Duration;

pub const CDN_URL: &str = "https://cdn.example/video.mp4";

/// A short, jitter-free ladder: 1s, 2s, 4s, capped at 8s, with a
/// permanent budget far below the transient one.
pub fn retry_policy() -> RetryPolicy {
    RetryPolicy {
        base: Duration::from_secs(1),
        max: Duration::from_secs(8),
        jitter: 0.0,
        transient_attempts: 8,
        permanent_attempts: 2,
        revive_after: Duration::from_secs(600),
    }
}

pub fn cdn_source() -> Source {
    Source::new(PostId::new("aa11"), CDN_URL.to_owned())
}
