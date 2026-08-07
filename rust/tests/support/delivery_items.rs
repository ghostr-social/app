//! Builders for focus updates fed to the delivery manager.

use rust_lib_ghostr::engine::{DeliveryKind, PostId, VideoMeta};
use rust_lib_ghostr::video::delivery_events::DeliveryCandidate;
use rust_lib_ghostr::video::delivery_events::{DeliveryFocus, FocusItem};

pub struct ItemSpec {
    pub id: &'static str,
    pub url: String,
    pub size: Option<u64>,
    pub duration_ms: Option<u64>,
}

pub fn progressive_item(spec: ItemSpec) -> FocusItem {
    FocusItem {
        post: PostId::new(spec.id),
        meta: VideoMeta {
            urls: vec![spec.url],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: spec.size,
            duration_ms: spec.duration_ms,
        },
    }
}

pub fn candidate(
    id: &'static str,
    url: &str,
    size: Option<u64>,
    discovered_at: u64,
) -> DeliveryCandidate {
    let item = progressive_item(ItemSpec {
        id,
        url: url.to_owned(),
        size,
        duration_ms: Some(1_000),
    });
    DeliveryCandidate {
        post: item.post,
        meta: item.meta,
        discovered_at,
    }
}

pub fn sized_item(id: &'static str, url: &str, size: u64, duration_ms: u64) -> FocusItem {
    progressive_item(ItemSpec {
        id,
        url: url.to_owned(),
        size: Some(size),
        duration_ms: Some(duration_ms),
    })
}

pub fn unsized_item(id: &'static str, url: &str) -> FocusItem {
    progressive_item(ItemSpec {
        id,
        url: url.to_owned(),
        size: None,
        duration_ms: None,
    })
}

pub fn unsized_mirrored_item(id: &'static str, first: &str, second: &str) -> FocusItem {
    let mut item = unsized_item(id, first);
    item.meta.urls.push(second.to_owned());
    item
}

pub fn focus_now(items: Vec<FocusItem>, current_index: usize, watch_ms: u64) -> DeliveryFocus {
    DeliveryFocus {
        items,
        current_index,
        watch_ms,
    }
}
