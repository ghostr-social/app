#![allow(dead_code)]

pub mod content_type;
pub mod ranged;
pub mod reject;
pub mod stall;

use ghostr_delivery::debug::network::NetworkThrottle;
use ghostr_net::outbound_media_client::MediaHttpRequests;
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

pub struct LocalMediaClient(reqwest::Client);

impl MediaHttpRequests for LocalMediaClient {
    fn get(&self, url: &str) -> anyhow::Result<reqwest::RequestBuilder> {
        Ok(self.0.get(url))
    }
}

pub fn media_client() -> LocalMediaClient {
    let client = reqwest::Client::builder()
        .no_proxy()
        .build()
        .expect("local media client");
    LocalMediaClient(client)
}

pub fn network() -> NetworkThrottle {
    NetworkThrottle::new()
}

/// A directory no other caller holds. The clock alone cannot promise
/// that: it repeats a nanosecond reading often enough that two fixtures
/// built in the same instant would share a root, so the process and a
/// per-call counter carry the uniqueness and the reading only separates
/// this run from an earlier one that left a directory behind.
pub fn temp_root(prefix: &str) -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    let sequence = NEXT.fetch_add(1, Ordering::Relaxed);
    let process = std::process::id();
    std::env::temp_dir().join(format!("{prefix}-{nonce}-{process}-{sequence}"))
}

pub fn store(root: PathBuf) -> PartialRangeStore {
    PartialRangeStore::with_capacity(
        root,
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    )
}

pub fn body() -> Vec<u8> {
    b"0123456789abcdef".to_vec()
}
