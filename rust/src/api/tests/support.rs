//! Shared builders for the API mapping and watcher tests.

use crate::api::delivery_types::{FfiFocusItem, FfiMediaDelivery};
use crate::discovery::content::parsing::ParsedVideoPost;
use crate::discovery::content::profiles::CreatorProfile;
use crate::engine::catalog::Catalog;
use crate::engine::{DeliveryKind, PostId, VideoMeta};
use core::sync::atomic::{AtomicU64, Ordering};
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

/// One parsed post with every optional field populated; the kind and
/// the addressable `d` identifier vary per test.
pub(crate) fn parsed_video_post(kind: u16, identifier: Option<&str>) -> ParsedVideoPost {
    ParsedVideoPost {
        event_id: "e1".to_owned(),
        author_pubkey: "a1".to_owned(),
        kind,
        identifier: identifier.map(str::to_owned),
        published_identifier: identifier.map(str::to_owned),
        created_at: 77,
        feed_sort_at: 77,
        repost: None,
        signed_event_json: None,
        is_protected: false,
        caption: "sunset ride".to_owned(),
        title: None,
        hashtags: vec!["sunset".to_owned()],
        dimensions: Some((608, 1080)),
        blurhash: Some("LKO2".to_owned()),
        thumbnail_url: Some("https://cdn.example/t.jpg".to_owned()),
        meta: VideoMeta {
            urls: vec!["https://cdn.example/v.mp4".to_owned()],
            delivery: DeliveryKind::Progressive,
            sha256: Some("ff".repeat(32)),
            size_bytes: Some(9),
            duration_ms: Some(2_000),
        },
        metadata_evidence: Vec::new(),
        renditions: Vec::new(),
    }
}

pub(crate) fn creator_profile() -> CreatorProfile {
    CreatorProfile {
        display_name: "Vera".to_owned(),
        handle: "@npub1vera".to_owned(),
        avatar_url: Some("https://cdn.example/a.png".to_owned()),
    }
}

pub(crate) fn ffi_item(id: &str, delivery: FfiMediaDelivery) -> FfiFocusItem {
    FfiFocusItem {
        post_id: id.to_owned(),
        urls: vec![format!("https://media.example/{id}.mp4")],
        delivery,
        sha256: Some("ab".repeat(32)),
        size_bytes: Some(16),
        duration_ms: Some(2_000),
        blurhash: None,
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
    static NEXT_STORE: AtomicU64 = AtomicU64::new(1);
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    let sequence = NEXT_STORE.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("{prefix}-{nonce}-{sequence}"));
    Arc::new(PartialRangeStore::with_capacity(
        root,
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    ))
}

pub(crate) async fn bind_store(store: &PartialRangeStore, id: &str, meta: &VideoMeta) {
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new(id), meta.clone());
    store
        .bind_representation(binding)
        .await
        .expect("test fixture precondition must hold");
}
