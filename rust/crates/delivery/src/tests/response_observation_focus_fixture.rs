use crate::delivery_events::{DeliveryFocus, FocusItem};
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

pub(super) fn replacement_focus() -> DeliveryFocus {
    DeliveryFocus::compatibility(
        vec![FocusItem {
            post: PostId::new("replacement"),
            meta: VideoMeta {
                urls: vec!["https://replacement.example/video.mp4".into()],
                delivery: DeliveryKind::Progressive,
                sha256: None,
                size_bytes: Some(1),
                duration_ms: Some(1),
            },
        }],
        0,
        0,
    )
}
