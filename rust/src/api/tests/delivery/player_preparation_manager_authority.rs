use crate::api::runtime::tracked_items::TrackedItems;
use crate::api::tests::support::{bind_store, sized_meta, temp_store};
use ghostr_delivery::cache_registry::{CacheRegistry, CacheStatus, CacheVideo};
use ghostr_delivery::delivery_events::{DeliveryFocus, FocusItem};
use ghostr_engine::{PostId, VideoMeta};
use ghostr_gateway::progressive::capabilities::ProgressiveCapabilities;
use ghostr_partial_store::partial_range_store::{PartialRangeStore, StoredMediaSnapshot};
use std::sync::Arc;

pub(super) struct SeededAuthority {
    pub(super) store: Arc<PartialRangeStore>,
    pub(super) tracked: TrackedItems,
    pub(super) cache: CacheRegistry,
    pub(super) capabilities: ProgressiveCapabilities,
    meta: VideoMeta,
    pub(super) representation: String,
    pub(super) asset: String,
}

impl SeededAuthority {
    pub(super) async fn new() -> Self {
        let meta = sized_meta(16, 2_000);
        let (store, snapshot) = seeded_store(&meta).await;
        let tracked = TrackedItems::new();
        tracked.insert("clip".to_owned(), meta.clone());
        let capabilities = ProgressiveCapabilities::production();
        let asset = capabilities
            .issue(&snapshot)
            .await
            .expect("test fixture precondition must hold")
            .as_str()
            .to_owned();
        Self {
            store,
            tracked,
            cache: seeded_cache(&meta),
            capabilities,
            meta,
            representation: snapshot
                .binding()
                .expect("test fixture precondition must hold")
                .representation()
                .fingerprint()
                .to_owned(),
            asset,
        }
    }

    pub(super) fn focus(&self) -> DeliveryFocus {
        DeliveryFocus::compatibility(
            vec![FocusItem {
                post: PostId::new("clip"),
                meta: self.meta.clone(),
            }],
            0,
            0,
        )
    }
}

async fn seeded_store(meta: &VideoMeta) -> (Arc<PartialRangeStore>, StoredMediaSnapshot) {
    let store = temp_store("ghostr-player-preparation-manager");
    bind_store(&store, "clip", meta).await;
    store
        .set_total_len("clip", 16)
        .await
        .expect("test fixture precondition must hold");
    store
        .write_range("clip", 0, &[7; 16])
        .await
        .expect("test fixture precondition must hold");
    let snapshot = store
        .media_snapshot("clip")
        .await
        .expect("test fixture precondition must hold");
    (store, snapshot)
}

fn seeded_cache(meta: &VideoMeta) -> CacheRegistry {
    let cache = CacheRegistry::new();
    cache.replace([CacheVideo {
        id: "clip".to_owned(),
        meta: meta.clone(),
        status: CacheStatus::Complete,
    }]);
    cache
}
