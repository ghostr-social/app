use crate::api::delivery_events_stream::{watch_delivery, EventOut};
use crate::api::delivery_types::FfiDeliveryEvent;
use crate::api::runtime::tracked_items::TrackedItems;
use crate::api::tests::support::temp_store;
use crate::engine::catalog::Catalog;
use crate::engine::{DeliveryKind, PostId, VideoMeta};
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
async fn watcher_never_labels_old_representation_bytes_as_new_metadata() {
    let store = temp_store("ghostr-api-watch-representation");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("clip"), meta("https://old.example/video"));
    store.bind_representation(binding).await.unwrap();
    store.set_total_len("clip", 16).await.unwrap();
    store.write_range("clip", 0, &[7; 16]).await.unwrap();
    let tracked = TrackedItems::new();
    tracked.insert("clip".to_owned(), meta("https://new.example/video"));
    let (sender, mut events) = mpsc::unbounded_channel();
    tokio::spawn(watch_delivery(
        ChannelOut(sender),
        store,
        SegmentedCache::new(),
        tracked,
    ));

    let event = tokio::time::timeout(Duration::from_secs(2), events.recv())
        .await
        .unwrap()
        .unwrap();

    assert!(!event.startable);
    assert_eq!(event.bytes_present, 0);
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
