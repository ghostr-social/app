//! Shared scaffolding for the gateway's routed tests: a temp root, a
//! local media client, and harnesses that put a real router behind them.

#![allow(dead_code)]

#[cfg(feature = "video-debug-web")]
pub mod commands;
#[cfg(feature = "video-debug-web")]
pub mod debug_clear;
pub mod delivery;
pub mod free_space;
pub mod hls_origin;
pub mod hls_prepared_origin;
pub mod hls_prepared_session;
#[cfg(feature = "video-debug-web")]
pub mod media_origin;
pub mod progressive;
mod progressive_capability;
pub mod progressive_delivery;
pub mod progressive_hls;
pub mod progressive_journey_item;
pub mod progressive_journey_origin;
pub mod progressive_journey_trace;
pub mod progressive_request;
pub mod raw_http;
pub mod request_gate_origin;

use core::sync::atomic::{AtomicU64, Ordering};
use ghostr_delivery::cache_registry::{CacheStatus, CacheVideo};
use ghostr_engine::VideoMeta;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use ghostr_net::outbound_media_client::MediaHttpRequests;
use reqwest::{Client, RequestBuilder};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// A directory no other caller holds. The clock alone cannot promise
/// that: it repeats a nanosecond reading often enough that two fixtures
/// built in the same instant would share a root, so the process and a
/// per-call counter carry the uniqueness and the reading only separates
/// this run from an earlier one that left a directory behind.
pub fn temp_directory(prefix: &str) -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    let sequence = NEXT.fetch_add(1, Ordering::Relaxed);
    let process = std::process::id();
    std::env::temp_dir().join(format!("{prefix}-{nonce}-{process}-{sequence}"))
}

struct LocalMediaHttpClient(Client);

impl MediaHttpRequests for LocalMediaHttpClient {
    fn get(&self, raw_url: &str) -> anyhow::Result<RequestBuilder> {
        Ok(self.0.get(raw_url))
    }
}

pub fn media_client() -> MediaRequestExecutor {
    let client = Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("local media client");
    MediaRequestExecutor::new(
        Arc::new(LocalMediaHttpClient(client)),
        MediaRequestLimits::try_new(4, 4).expect("valid test fixture"),
    )
}

pub fn cache_video(id: impl Into<String>, meta: VideoMeta) -> CacheVideo {
    CacheVideo {
        id: id.into(),
        meta,
        status: CacheStatus::Ready,
    }
}

pub fn progressive_startup() -> ghostr_engine::media_timeline::StartupFootprint {
    let bytes = progressive_journey_origin::fixture::supported_mp4();
    ghostr_engine::media_timeline::parse_mp4_segments(&[
        ghostr_engine::media_timeline::MediaSegment::new(0, &bytes),
    ])
    .expect("progressive fixture timeline")
    .startup_footprint()
    .expect("progressive fixture startup")
}

pub fn progressive_fixture_bytes() -> Vec<u8> {
    progressive_journey_origin::fixture::progressive_mp4()
}
