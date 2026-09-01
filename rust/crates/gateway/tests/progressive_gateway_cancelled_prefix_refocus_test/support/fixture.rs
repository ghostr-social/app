use crate::gateway_fixture::progressive_delivery::ProgressiveDeliveryHarness;
use crate::support::ControlledOrigin;
use ghostr_delivery::delivery_events::FocusItem;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::time::Duration;

pub const TOTAL: u64 = 293_999;

pub fn roster(origin: &ControlledOrigin) -> Vec<FocusItem> {
    (0..7).map(|index| item(origin, index)).collect()
}

pub async fn seed_ready_ranges(
    harness: &ProgressiveDeliveryHarness,
    items: &[FocusItem],
    bytes: &[u8],
) {
    assert_eq!(bytes.len(), TOTAL as usize, "real progressive fixture size");
    for (index, item) in items.iter().enumerate() {
        seed(harness, item, (index < 6).then_some(bytes)).await;
    }
}

pub async fn wait_for_current_authority(harness: &ProgressiveDeliveryHarness, post: &str) {
    let current = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let snapshot = harness
                .delivery
                .store
                .media_snapshot(post)
                .await
                .expect("authority snapshot");
            let allowed = snapshot
                .binding()
                .is_some_and(|binding| harness.delivery.cache.allows_binding(post, binding));
            if allowed && snapshot.total_len().is_some() {
                return;
            }
            tokio::task::yield_now().await;
        }
    })
    .await;
    assert!(
        current.is_ok(),
        "p6 cache never allowed its current stored representation"
    );
}

fn item(origin: &ControlledOrigin, index: usize) -> FocusItem {
    let id = format!("p{index}");
    FocusItem {
        post: PostId::new(id.clone()),
        meta: VideoMeta {
            urls: vec![origin.url_for(&id)],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(TOTAL),
            duration_ms: Some(4_000),
        },
    }
}

async fn seed(harness: &ProgressiveDeliveryHarness, item: &FocusItem, bytes: Option<&[u8]>) {
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(item.post.clone(), item.meta.clone());
    let store = &harness.delivery.store;
    store.bind_representation(binding).await.expect("binding");
    store
        .set_total_len(item.post.as_str(), TOTAL)
        .await
        .expect("total");
    if let Some(bytes) = bytes {
        store
            .write_range(item.post.as_str(), 0, bytes)
            .await
            .expect("ready bytes");
    }
}
