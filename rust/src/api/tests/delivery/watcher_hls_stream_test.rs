use crate::api::delivery_events_stream::{watch_delivery, DeliveryWatchContext, EventOut};
use crate::api::delivery_types::{FfiDeliveryEvent, FfiDeliveryEventKind};
use crate::api::runtime::tracked_items::TrackedItems;
use crate::api::tests::support::temp_store;
use crate::engine::{DeliveryKind, VideoMeta};
use ghostr_delivery::cache_registry::CacheRegistry;
use ghostr_delivery::segmented::SegmentedCache;
use std::time::Duration;
use tokio::sync::mpsc;

struct ChannelOut(mpsc::UnboundedSender<FfiDeliveryEvent>);

impl EventOut for ChannelOut {
    fn send(&self, event: FfiDeliveryEvent) -> bool {
        self.0.send(event).is_ok()
    }
}

#[tokio::test]
async fn includes_hls_posts_in_the_readiness_baseline() {
    let tracked = TrackedItems::new();
    tracked.insert("stream".to_owned(), hls_meta());
    let context = DeliveryWatchContext::new(
        temp_store("ghostr-api-hls-watch"),
        SegmentedCache::new(),
        tracked,
        CacheRegistry::new(),
    );
    let (sender, mut events) = mpsc::unbounded_channel();
    tokio::spawn(watch_delivery(ChannelOut(sender), context));

    let event = tokio::time::timeout(Duration::from_secs(10), events.recv())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(event.post_id, "stream");
    assert_eq!(event.kind, FfiDeliveryEventKind::Readiness);
    assert!(!event.startable);
    assert_eq!(event.eta_ms, None);
}

fn hls_meta() -> VideoMeta {
    VideoMeta {
        urls: vec!["https://media.example/stream.m3u8".to_owned()],
        delivery: DeliveryKind::Hls,
        sha256: None,
        size_bytes: None,
        duration_ms: Some(2_000),
    }
}
