use crate::api::delivery_events_stream::{watch_delivery, DeliveryWatchContext, EventOut};
use crate::api::delivery_types::FfiDeliveryEvent;
use crate::api::runtime::tracked_items::TrackedItems;
use crate::api::tests::delivery::selected_rendition_fixture::selected_rendition;
use crate::api::tests::support::temp_store;
use crate::engine::catalog::Catalog;
use crate::engine::{DeliveryKind, PostId, VideoMeta};
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
async fn watcher_never_labels_old_representation_bytes_as_new_metadata() {
    let store = temp_store("ghostr-api-watch-representation");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("clip"), meta("https://old.example/video"));
    store
        .bind_representation(binding)
        .await
        .expect("test fixture precondition must hold");
    store
        .set_total_len("clip", 16)
        .await
        .expect("test fixture precondition must hold");
    store
        .write_range("clip", 0, &[7; 16])
        .await
        .expect("test fixture precondition must hold");
    let tracked = TrackedItems::new();
    let current = meta("https://new.example/video");
    tracked.insert("clip".to_owned(), current.clone());
    let cache = CacheRegistry::new();
    cache.replace([cache_video(current)]);
    let (sender, mut events) = mpsc::unbounded_channel();
    tokio::spawn(watch_delivery(
        ChannelOut(sender),
        DeliveryWatchContext::new(store, SegmentedCache::new(), tracked, cache),
    ));

    let event = next(&mut events).await;

    assert!(!event.startable);
    assert_eq!(event.bytes_present, 0);
}

#[tokio::test]
async fn watcher_attributes_selected_rendition_bytes_to_the_advertised_feed_item() {
    let store = temp_store("ghostr-api-watch-selected-rendition");
    let rendition = selected_rendition("clip");
    store
        .bind_representation(rendition.binding)
        .await
        .expect("test fixture precondition must hold");
    store
        .write_range("clip", 0, &[7; 16])
        .await
        .expect("test fixture precondition must hold");
    let tracked = TrackedItems::new();
    tracked.insert("clip".to_owned(), rendition.advertised);
    let cache = CacheRegistry::new();
    cache.insert("clip");
    let (sender, mut events) = mpsc::unbounded_channel();
    tokio::spawn(watch_delivery(
        ChannelOut(sender),
        DeliveryWatchContext::new(store, SegmentedCache::new(), tracked, cache.clone()),
    ));

    let baseline = next(&mut events).await;
    assert!(!baseline.startable);
    assert_eq!(baseline.bytes_present, 0);
    assert_eq!(baseline.total_bytes, Some(64));
    cache.replace([cache_video(rendition.selected)]);
    let event = next(&mut events).await;

    assert!(event.startable);
    assert_eq!(event.bytes_present, 16);
    assert_eq!(event.total_bytes, Some(16));
}

async fn next(events: &mut mpsc::UnboundedReceiver<FfiDeliveryEvent>) -> FfiDeliveryEvent {
    tokio::time::timeout(Duration::from_secs(2), events.recv())
        .await
        .expect("test fixture precondition must hold")
        .expect("test fixture precondition must hold")
}

fn cache_video(meta: VideoMeta) -> CacheVideo {
    CacheVideo {
        id: "clip".to_owned(),
        meta,
        status: CacheStatus::Complete,
    }
}

fn meta(url: &str) -> VideoMeta {
    VideoMeta {
        urls: vec![url.to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(16),
        duration_ms: Some(2_000),
    }
}
