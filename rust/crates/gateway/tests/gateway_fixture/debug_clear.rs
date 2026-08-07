use ghostr_delivery::debug_feed::DebugFeedItem;
use ghostr_engine::{DeliveryKind, VideoMeta};

pub fn progressive_meta() -> VideoMeta {
    VideoMeta {
        urls: vec!["https://media.example/stored.mp4".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(16),
        duration_ms: Some(1_000),
    }
}

pub fn hls_item() -> DebugFeedItem {
    DebugFeedItem {
        id: "live".to_owned(),
        event_id: "event-live".to_owned(),
        title: None,
        creator: "creator".to_owned(),
        created_at: 1,
        meta: VideoMeta {
            delivery: DeliveryKind::Hls,
            urls: vec!["https://media.example/live.m3u8".to_owned()],
            sha256: None,
            size_bytes: None,
            duration_ms: None,
        },
    }
}
