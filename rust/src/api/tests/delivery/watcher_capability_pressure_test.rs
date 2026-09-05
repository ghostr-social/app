use crate::api::delivery_events_stream::{watch_delivery, DeliveryWatchContext, EventOut};
use crate::api::delivery_types::FfiDeliveryEvent;
use crate::api::runtime::tracked_items::TrackedItems;
use crate::api::tests::support::{bind_store, sized_meta, temp_store};
use core::time::Duration;
use ghostr_delivery::cache_registry::{CacheRegistry, CacheStatus, CacheVideo};
use ghostr_delivery::segmented::SegmentedCache;
use ghostr_gateway::progressive::capabilities::{
    ProgressiveCapabilities, ProgressiveCapabilityLimits,
};
use tokio::sync::mpsc;

struct ChannelOut(mpsc::UnboundedSender<FfiDeliveryEvent>);

impl EventOut for ChannelOut {
    fn send(&self, event: FfiDeliveryEvent) -> bool {
        self.0.send(event).is_ok()
    }
}

#[tokio::test]
async fn observing_catalog_items_does_not_evict_active_playback_authority() {
    let store = temp_store("ghostr-live-catalog-capabilities");
    let meta = sized_meta(16, 2_000);
    let tracked = TrackedItems::new();
    let cache = CacheRegistry::new();
    let mut entries = Vec::new();
    for id in ["catalog-only", "playing"] {
        bind_store(&store, id, &meta).await;
        store.set_total_len(id, 16).await.expect("stored length");
        tracked.insert(id.to_owned(), meta.clone());
        entries.push(CacheVideo {
            id: id.to_owned(),
            meta: meta.clone(),
            status: CacheStatus::Ready,
        });
    }
    cache.replace(entries);
    let capabilities = ProgressiveCapabilities::new(
        ProgressiveCapabilityLimits::new(1, Duration::from_secs(60)).expect("limits"),
    );
    let snapshot = store
        .media_snapshot("playing")
        .await
        .expect("active snapshot");
    let active = capabilities
        .issue(&snapshot)
        .await
        .expect("active capability");
    let (sender, mut events) = mpsc::unbounded_channel();
    let watcher = tokio::spawn(watch_delivery(
        ChannelOut(sender),
        DeliveryWatchContext::new(store, SegmentedCache::new(), tracked, cache)
            .with_capabilities(capabilities.clone()),
    ));
    for _ in 0..2 {
        tokio::time::timeout(Duration::from_secs(3), events.recv())
            .await
            .expect("watcher deadline")
            .expect("watcher event");
    }
    watcher.abort();
    assert!(
        capabilities
            .authorizes(active.as_str(), "playing", &snapshot)
            .await,
        "a read-only catalog observation evicted active playback"
    );
}
