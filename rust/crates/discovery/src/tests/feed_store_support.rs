//! Parsed-post builders for the feed store's own tests: posts identified
//! by the second they were created, so a page reads as a time window.

use crate::event_parsing::ParsedVideoPost;
use ghostr_engine::{DeliveryKind, VideoMeta};

/// One fetched page: `count` posts newest first, one second apart,
/// starting at `newest_at`.
pub fn page(newest_at: u64, count: u64) -> Vec<ParsedVideoPost> {
    (0..count).map(|step| post(newest_at - step)).collect()
}

/// A minimal parsed post whose identity is its creation second.
pub fn post(created_at: u64) -> ParsedVideoPost {
    ParsedVideoPost {
        event_id: format!("{created_at:064}"),
        author_pubkey: "a".repeat(64),
        kind: 1,
        identifier: None,
        published_identifier: None,
        created_at,
        caption: String::new(),
        title: None,
        hashtags: Vec::new(),
        dimensions: None,
        blurhash: None,
        thumbnail_url: None,
        meta: VideoMeta {
            urls: vec!["https://example.com/video.mp4".to_owned()],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: None,
            duration_ms: None,
        },
    }
}
