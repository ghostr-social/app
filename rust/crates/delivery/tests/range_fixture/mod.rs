#![allow(dead_code)]

pub mod cancellation;
pub mod content_type;
pub mod ranged;
pub mod reject;
pub mod short;
pub mod stall;

use ghostr_delivery::chunk::cancel::CancelToken;
use ghostr_delivery::chunk::downloader::{
    download_chunk_observed, ChunkResult, ChunkSink, ChunkSpec, DownloadTraffic,
};
use ghostr_delivery::debug::network::NetworkThrottle;
use ghostr_engine::adaptive::RetrievalRequest;
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::ByteRange;
use ghostr_net::outbound_media_client::MediaHttpRequests;
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
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

pub fn range_request(bytes: ByteRange) -> RetrievalRequest {
    RetrievalRequest::FetchRange {
        bytes,
        promotion: None,
    }
}

pub async fn download_chunk_throttled(
    spec: &ChunkSpec<'_>,
    sink: &ChunkSink<'_>,
    stats: &mut HostStats,
    cancel: &CancelToken,
    network: &NetworkThrottle,
) -> anyhow::Result<ChunkResult> {
    download_chunk_observed(spec, sink, stats, cancel, network, &mut IgnoreTraffic).await
}

struct IgnoreTraffic;

impl DownloadTraffic for IgnoreTraffic {
    fn opened(&mut self, _ttfb: Duration) {}

    fn wrote(&mut self, _bytes: u64) {}
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
