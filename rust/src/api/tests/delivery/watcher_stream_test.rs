use crate::api::delivery_events_stream::{watch_delivery, EventOut};
use crate::api::delivery_types::{FfiDeliveryEvent, FfiDeliveryEventKind};
use crate::api::runtime::tracked_items::TrackedItems;
use crate::api::tests::support::{bind_store, sized_meta, temp_store};
use crate::engine::{DeliveryKind, VideoMeta};
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
async fn streams_readiness_once_the_head_bytes_land() {
    let store = temp_store("ghostr-api-watch");
    let tracked = TrackedItems::new();
    let meta = sized_meta(16, 2_000);
    bind_store(&store, "clip", &meta).await;
    tracked.insert("clip".to_owned(), meta);
    let (sender, mut events) = mpsc::unbounded_channel();
    tokio::spawn(watch_delivery(
        ChannelOut(sender),
        store.clone(),
        SegmentedCache::new(),
        tracked,
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

#[tokio::test]
async fn includes_hls_posts_in_the_readiness_baseline() {
    let tracked = TrackedItems::new();
    tracked.insert("stream".to_owned(), hls_meta());
    let (sender, mut events) = mpsc::unbounded_channel();
    tokio::spawn(watch_delivery(
        ChannelOut(sender),
        temp_store("ghostr-api-hls-watch"),
        SegmentedCache::new(),
        tracked,
    ));

    let event = recv(&mut events).await;

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
