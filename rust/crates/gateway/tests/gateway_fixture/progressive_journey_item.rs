use ghostr_delivery::delivery_events::FocusItem;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

pub fn unknown_item(post: &str, url: &str) -> FocusItem {
    FocusItem {
        post: PostId::new(post),
        meta: VideoMeta {
            urls: vec![url.to_owned()],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: None,
            duration_ms: None,
        },
    }
}
