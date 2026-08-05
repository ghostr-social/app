//! Builders for trending-hashtag contract fixtures.

use crate::discovery::event_parsing::ParsedVideoPost;
use crate::engine::{DeliveryKind, VideoMeta};

/// A minimal parsed post carrying exactly the given hashtags.
pub fn post_with_hashtags(event_id: &str, hashtags: &[&str]) -> ParsedVideoPost {
    ParsedVideoPost {
        event_id: event_id.to_owned(),
        author_pubkey: "a".repeat(64),
        kind: 1,
        identifier: None,
        published_identifier: None,
        created_at: 1,
        caption: String::new(),
        title: None,
        hashtags: hashtags.iter().map(|tag| (*tag).to_owned()).collect(),
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
