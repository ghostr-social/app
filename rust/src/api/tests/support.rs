//! Shared builders for the API mapping and watcher tests.

use crate::api::delivery_types::FfiFocusItem;
use crate::engine::{DeliveryKind, VideoMeta};
use crate::video::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

pub(crate) fn ffi_item(id: &str, delivery: &str) -> FfiFocusItem {
    FfiFocusItem {
        post_id: id.to_owned(),
        urls: vec![format!("https://media.example/{id}.mp4")],
        delivery: delivery.to_owned(),
        sha256: Some("ab".repeat(32)),
        size_bytes: Some(16),
        duration_ms: Some(2_000),
    }
}

/// Sixteen bytes over two seconds: the whole file is the head under
/// default parameters, so covering it makes the post startable.
pub(crate) fn sized_meta(size_bytes: u64, duration_ms: u64) -> VideoMeta {
    VideoMeta {
        urls: vec!["https://media.example/clip.mp4".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(size_bytes),
        duration_ms: Some(duration_ms),
    }
}

pub(crate) fn temp_store(prefix: &str) -> Arc<PartialRangeStore> {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("{prefix}-{nonce}"));
    Arc::new(PartialRangeStore::new(root, Arc::new(Mutex::new(0))))
}
