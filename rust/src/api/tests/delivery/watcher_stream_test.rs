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
async fn streams_readiness_once_the_head_bytes_land() {
    let store = temp_store("ghostr-api-watch");
    let tracked = TrackedItems::new();
    let meta = sized_meta(16, 2_000);
    bind_store(&store, "clip", &meta).await;
    tracked.insert("clip".to_owned(), meta.clone());
    let cache = CacheRegistry::new();
    cache.replace([CacheVideo {
        id: "clip".to_owned(),
        meta,
        status: CacheStatus::Ready,
    }]);
    let (sender, mut events) = mpsc::unbounded_channel();
    tokio::spawn(watch_delivery(
        ChannelOut(sender),
        DeliveryWatchContext::new(
            std::sync::Arc::clone(&store),
            SegmentedCache::new(),
            tracked,
            cache,
        ),
    ));

    let first = recv(&mut events).await;
    assert_eq!(first.kind, FfiDeliveryEventKind::Readiness);
    assert!(!first.startable);

    store.set_total_len("clip", 16).await.expect("total length");
    store
        .write_range("clip", 0, &[7u8; 16])
        .await
        .expect("write");

    let ready = wait_for_startable(&mut events).await;
    assert_eq!(ready.kind, FfiDeliveryEventKind::Readiness);
    assert_eq!(ready.bytes_present, 16);
    assert_eq!(ready.total_bytes, Some(16));
}

async fn recv(events: &mut mpsc::UnboundedReceiver<FfiDeliveryEvent>) -> FfiDeliveryEvent {
    tokio::time::timeout(Duration::from_secs(10), events.recv())
        .await
        .expect("event deadline")
        .expect("open stream")
}

async fn wait_for_startable(
    events: &mut mpsc::UnboundedReceiver<FfiDeliveryEvent>,
) -> FfiDeliveryEvent {
    loop {
        let event = recv(events).await;
        if event.startable {
            return event;
        }
    }
}
