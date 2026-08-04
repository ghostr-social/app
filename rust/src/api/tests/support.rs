//! Shared builders for the API mapping and watcher tests.

use crate::api::delivery_types::FfiFocusItem;
use crate::discovery::event_parsing::ParsedVideoPost;
use crate::discovery::profile_store::CreatorProfile;
use crate::engine::{DeliveryKind, VideoMeta};
use crate::video::partial_range_store::PartialRangeStore;
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
        created_at: 77,
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
    }
}

pub(crate) fn creator_profile() -> CreatorProfile {
    CreatorProfile {
        display_name: "Vera".to_owned(),
        handle: "@npub1vera".to_owned(),
        avatar_url: Some("https://cdn.example/a.png".to_owned()),
    }
}

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
