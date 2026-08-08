#![cfg(feature = "video-debug-web")]

use ghostr_delivery::debug::feed::{DebugFeed, DebugFeedItem, DebugFeedStage};
use ghostr_delivery::delivery_events::{command_channel, DeliveryCommand};
use ghostr_engine::{DeliveryKind, VideoMeta};

fn item(id: &str) -> DebugFeedItem {
    DebugFeedItem {
        id: id.to_owned(),
        event_id: format!("event-{id}"),
        title: None,
        creator: "creator".to_owned(),
        created_at: 1,
        meta: VideoMeta {
            urls: vec![format!("https://media.example/{id}.mp4")],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(10),
            duration_ms: Some(1_000),
        },
    }
}

#[tokio::test]
async fn cleared_events_stay_hidden_while_new_discovery_keeps_flowing() {
    let (delivery, mut commands) = command_channel();
    let feed = DebugFeed::new(delivery, Vec::new());
    feed.publish(1, DebugFeedStage::Settled, vec![item("old")]);
    commands.receivers().0.recv().await.expect("initial focus");

    feed.clear();
    let DeliveryCommand::Focus(cleared) = commands.receivers().0.recv().await.expect("clear focus")
    else {
        panic!("expected focus");
    };
    assert!(cleared.items.is_empty());

    feed.publish(2, DebugFeedStage::Settled, vec![item("old"), item("new")]);

    assert_eq!(feed.snapshot().discovered_count, 1);
    assert!(feed.metadata("old").is_none());
    assert!(feed.metadata("new").is_some());
}
