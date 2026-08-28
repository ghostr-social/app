#![expect(dead_code, reason = "shared fixture APIs vary by integration scenario")]

pub mod cancellation;
pub mod content_type;
pub mod download;
pub mod header_failure;
mod media_client;
pub mod promoted_stall;
pub mod ranged;
pub mod reject;
pub mod short;
pub mod stall;

use core::sync::atomic::{AtomicU64, Ordering};
use ghostr_delivery::debug::network::NetworkThrottle;
use ghostr_engine::adaptive::RetrievalRequest;
use ghostr_engine::ByteRange;
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

#[expect(
    unused_imports,
    reason = "shared fixture APIs vary by integration scenario"
)]
pub use download::{
    context, download_chunk_throttled, download_chunk_with_traffic, DownloadContext,
    ObservationTraffic,
};

pub fn media_client() -> ghostr_net::media_request_executor::MediaRequestExecutor {
    media_client::media_client()
}

pub fn raw_media_client() -> Arc<dyn ghostr_net::outbound_media_client::MediaHttpRequests> {
    media_client::raw_media_client()
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

pub fn range_profile(bytes: u64) -> ghostr_engine::origin_model::OriginAttemptProfile {
    use ghostr_engine::origin_model::{MediaClass, OriginRequestProfile, RequestMethod};
    let profile =
        OriginRequestProfile::new(RequestMethod::RangeGet, bytes, MediaClass::ProgressiveMp4);
    ghostr_engine::origin_model::OriginAttemptProfile::new(profile)
}

pub fn whole_profile(bytes: u64) -> ghostr_engine::origin_model::OriginAttemptProfile {
    use ghostr_engine::origin_model::{MediaClass, OriginRequestProfile, RequestMethod};
    let profile = OriginRequestProfile::new(RequestMethod::FullGet, bytes, MediaClass::WholeObject);
    ghostr_engine::origin_model::OriginAttemptProfile::new(profile)
}

pub fn head_profile() -> ghostr_engine::origin_model::OriginAttemptProfile {
    use ghostr_engine::origin_model::{MediaClass, OriginRequestProfile, RequestMethod};
    let profile = OriginRequestProfile::new(RequestMethod::Head, 0, MediaClass::Unknown);
    ghostr_engine::origin_model::OriginAttemptProfile::new(profile)
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
