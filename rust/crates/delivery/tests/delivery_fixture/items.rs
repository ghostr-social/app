//! Builders for focus updates fed to the delivery manager.

use ghostr_delivery::delivery_events::DeliveryCandidate;
use ghostr_delivery::delivery_events::{DeliveryFocus, FocusItem};
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use ghostr_partial_store::partial_range_store::PartialRangeStore;

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
    DeliveryFocus::compatibility(items, current_index, watch_ms)
}

pub async fn seed_range(store: &PartialRangeStore, item: &FocusItem, offset: u64, bytes: &[u8]) {
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(item.post.clone(), item.meta.clone());
    let source = item.meta.urls.first().expect("fixture source");
    let identity = binding.transfer(source).expect("fixture identity");
    store.bind_representation(binding).await.unwrap();
    store.select_transfer(identity.clone());
    assert!(store
        .write_range_for_transfer_if_current(&identity, offset, bytes)
        .await
        .unwrap());
}
