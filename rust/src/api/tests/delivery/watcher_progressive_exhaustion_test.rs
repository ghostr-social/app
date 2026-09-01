use crate::api::delivery_events_stream::{watch_delivery, DeliveryWatchContext, EventOut};
use crate::api::delivery_types::{FfiDeliveryEvent, FfiDeliveryEventKind};
use crate::api::runtime::tracked_items::TrackedItems;
use crate::api::tests::support::{bind_store, sized_meta, temp_store};
use core::time::Duration;
use ghostr_delivery::cache_registry::{CacheRegistry, CacheStatus, CacheVideo};
use ghostr_delivery::segmented::SegmentedCache;
use tokio::sync::mpsc;

struct ChannelOut(mpsc::UnboundedSender<FfiDeliveryEvent>);

impl EventOut for ChannelOut {
    fn send(&self, event: FfiDeliveryEvent) -> bool {
        self.0.send(event).is_ok()
    }
}

#[tokio::test]
async fn all_sources_failed_progressive_item_emits_failed() {
    let store = temp_store("ghostr-api-progressive-exhaustion");
    let tracked = TrackedItems::new();
    let meta = sized_meta(16, 2_000);
    bind_store(&store, "clip", &meta).await;
    tracked.insert("clip".to_owned(), meta.clone());
    let cache = CacheRegistry::new();
    cache.replace([video(meta.clone(), CacheStatus::Ready)]);
    let (sender, mut events) = mpsc::unbounded_channel();
    tokio::spawn(watch_delivery(
        ChannelOut(sender),
        DeliveryWatchContext::new(store, SegmentedCache::new(), tracked, cache.clone()),
    ));

    assert_eq!(
        next(&mut events).await.kind,
        FfiDeliveryEventKind::Readiness
    );
    cache.replace([video(meta, CacheStatus::Failed)]);

    let failed = next(&mut events).await;
    assert_eq!(failed.kind, FfiDeliveryEventKind::Failed);
    assert!(!failed.startable);
    assert_eq!(failed.detail.as_deref(), Some("all sources failed"));
}

fn video(meta: crate::engine::VideoMeta, status: CacheStatus) -> CacheVideo {
    CacheVideo {
        id: "clip".to_owned(),
        meta,
        status,
    }
}

async fn next(events: &mut mpsc::UnboundedReceiver<FfiDeliveryEvent>) -> FfiDeliveryEvent {
    tokio::time::timeout(Duration::from_secs(2), events.recv())
        .await
        .expect("delivery event deadline")
        .expect("open delivery stream")
}
