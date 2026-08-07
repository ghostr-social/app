//! A whole-blob cache over a temp directory, plus the identity and
//! client the download path needs to reach a fixture origin.

#![allow(dead_code)]

pub mod raw_http;

use ghostr_media_model::native_models::NativeVideoCacheKey;
use ghostr_media_store::native_cache::{
    prepare_native_cache_directory, CachedVideo, NativeVideoCache,
};
use ghostr_net::outbound_media_client::MediaHttpClient;
use raw_http::spawn_raw_server;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

pub const VIDEO_DIGEST: &str = "0cab1c9617404faf2b24e221e189ca5945813e14d3f766345b09ca13bbe28ffc";
pub const VIDEO_RESPONSE: &[u8] =
    b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nvideo";

pub struct NativeCacheHarness {
    pub cache: NativeVideoCache,
    pub directory: PathBuf,
    pub used_bytes: Arc<Mutex<u64>>,
}

impl NativeCacheHarness {
    pub fn new(prefix: &str) -> Self {
        let directory = temp_directory(prefix);
        prepare_native_cache_directory(&directory).expect("prepare cache");
        let used_bytes = Arc::new(Mutex::new(0));
        Self {
            cache: NativeVideoCache::new(directory.clone(), 20, used_bytes.clone()),
            directory,
            used_bytes,
        }
    }

    pub async fn download(
        &self,
        key: &NativeVideoCacheKey,
        response: &'static [u8],
        expected: Option<&str>,
    ) -> CachedVideo {
        let (url, request) = spawn_raw_server(response).await;
        let cached = self
            .cache
            .download(&media_client(), key, &url, expected)
            .await
            .expect("cached video");
        request.await.expect("upstream request");
        cached
    }
}

pub fn advertised_key() -> NativeVideoCacheKey {
    NativeVideoCacheKey::AdvertisedDigest(VIDEO_DIGEST.to_owned())
}

pub fn video_id() -> String {
    "a".repeat(64)
}

pub fn video_cache_key() -> NativeVideoCacheKey {
    NativeVideoCacheKey::UrlDerived(video_id())
}

pub fn video_cache_file_id() -> String {
    video_cache_key().storage_id().expect("cache file id")
}

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
