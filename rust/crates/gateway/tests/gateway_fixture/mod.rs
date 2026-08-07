//! Shared scaffolding for the gateway's routed tests: a temp root, a
//! trusted media client, the downloads a proxied request resolves
//! against, and the harnesses that put a real router behind them.

#![allow(dead_code)]

#[cfg(feature = "video-debug-web")]
pub mod debug_clear;
pub mod delivery;
pub mod free_space;
pub mod progressive;
mod progressive_request;
pub mod raw_http;

use ghostr_media_model::native_models::{
    NativeEventIdentity, NativeUserData, NativeVideo, NativeVideoDelivery, NativeVideoDownload,
};
use ghostr_net::outbound_media_client::MediaHttpClient;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
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

pub fn media_client() -> MediaHttpClient {
    MediaHttpClient::trusted().expect("trusted media client")
}

pub fn video_id() -> String {
    "a".repeat(64)
}

pub fn native_download(url: &str) -> NativeVideoDownload {
    NativeVideoDownload::new(video_id(), native_video(url), event_identity())
}

fn native_video(url: &str) -> NativeVideo {
    NativeVideo {
        id: video_id(),
        expected_digest: None,
        fallback_urls: Vec::new(),
        user: NativeUserData {
            npub: Some("npub1author".to_owned()),
            name: Some("Ghost".to_owned()),
            profile_picture: Some("https://media.example/avatar.png".to_owned()),
        },
        title: "Relay clip".to_owned(),
        song_name: "Original sound".to_owned(),
        comments: "4".to_owned(),
        likes: "12".to_owned(),
        url: url.to_owned(),
        delivery: NativeVideoDelivery::Progressive,
    }
}

fn event_identity() -> NativeEventIdentity {
    NativeEventIdentity {
        event_id: "event-id".to_owned(),
        author_public_key_hex: "author-key".to_owned(),
        kind: 22,
        identifier: None,
        created_at: 42,
        content: "Relay clip".to_owned(),
        hashtags: Vec::new(),
    }
}
