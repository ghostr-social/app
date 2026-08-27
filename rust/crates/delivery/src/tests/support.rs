use core::sync::atomic::{AtomicU64, Ordering};
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::adaptive::RetrievalRequest;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::scheduling::RangeRequest;
use ghostr_engine::{ByteRange, ChunkId, DeliveryKind, PostId, VideoMeta};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) mod pressure;
mod queue;
mod request_profile;
pub(crate) use queue::{active_hosts, planned_queue, planned_transfer, transfer_posts};
pub(crate) use request_profile::{range_profile, whole_profile};

static NEXT_TEMP_DIRECTORY: AtomicU64 = AtomicU64::new(1);

pub(crate) fn temp_directory(prefix: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    let sequence = NEXT_TEMP_DIRECTORY.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("{prefix}-{nonce}-{sequence}"));
    std::fs::create_dir_all(&path).expect("create test directory");
    path
}

pub(crate) fn transfer_identity(post: &PostId, url: &str) -> TransferIdentity {
    let mut catalog = Catalog::new();
    catalog.upsert(
        post.clone(),
        VideoMeta {
            urls: vec![url.to_owned()],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(1),
            duration_ms: Some(1),
        },
    );
    catalog.transfer_identity(post, url).expect("test source")
}

pub(crate) fn chunk_request(chunk: ChunkId, authority: PreemptionAuthority) -> RangeRequest {
    RangeRequest {
        chunk,
        authority,
        score: 1.0,
        contiguous_depth_bytes: 0,
    }
}

pub(crate) fn range_retrieval(bytes: ByteRange) -> RetrievalRequest {
    RetrievalRequest::FetchRange {
        bytes,
        promotion: None,
    }
}
