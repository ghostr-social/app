use crate::engine::{DeliveryKind, PostId, VideoMeta};

pub fn progressive_meta(size_bytes: Option<u64>, duration_ms: Option<u64>) -> VideoMeta {
    VideoMeta {
        urls: vec!["https://host.example/video.mp4".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes,
        duration_ms,
    }
}

pub fn ids(raw: &[&str]) -> Vec<PostId> {
    raw.iter().map(|value| PostId::new(*value)).collect()
}
