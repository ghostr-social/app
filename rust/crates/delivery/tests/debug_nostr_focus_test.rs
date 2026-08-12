#![cfg(feature = "video-debug-web")]

mod command_fixture;

use command_fixture::next_control;
use ghostr_delivery::debug::feed::{DebugFeed, DebugFeedItem, DebugFeedStage, DebugRelaySnapshot};
use ghostr_delivery::delivery_events::{command_channel, DeliveryCommand};
use ghostr_engine::{DeliveryKind, VideoMeta};

fn item(id: &str) -> DebugFeedItem {
    DebugFeedItem {
        id: id.to_owned(),
        event_id: format!("event-{id}"),
        title: Some(format!("Video {id}")),
        creator: format!("Creator {id}"),
        created_at: 42,
        meta: VideoMeta {
            urls: vec![format!("https://media.example/{id}.mp4")],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(1_000),
            duration_ms: Some(10_000),
        },
    }
}

#[tokio::test]
async fn nostr_feed_drives_delivery_focus_and_selection() {
    let (delivery, mut commands) = command_channel();
    let feed = DebugFeed::new(delivery, vec!["wss://relay.example".to_owned()]);

    feed.publish(7, DebugFeedStage::Settled, vec![item("a"), item("b")]);
    let DeliveryCommand::Focus(initial) = next_control(&mut commands).await else {
        panic!("expected focus");
    };
    assert_eq!(initial.items.len(), 2);
    assert_eq!(initial.current_index, 0);
    assert_eq!(feed.snapshot().discovered_count, 2);
    assert_eq!(feed.snapshot().relays[0].status, "initializing");
    assert_eq!(feed.metadata("a").expect("metadata").event_id, "event-a");
    feed.update_relays(vec![DebugRelaySnapshot {
        url: "wss://relay.example".to_owned(),
        status: "connected".to_owned(),
    }]);
    assert_eq!(feed.snapshot().relays[0].status, "connected");

    feed.select("b").expect("select discovered video");
    let DeliveryCommand::Focus(selected) = next_control(&mut commands).await else {
        panic!("expected focus");
    };
    assert_eq!(selected.current_index, 1);
    assert_eq!(feed.snapshot().current_id.as_deref(), Some("b"));
    assert!(feed.select("missing").is_err());
}

#[tokio::test]
async fn feed_revisions_retain_selection_and_can_clear_the_window() {
    let (delivery, mut commands) = command_channel();
    let feed = DebugFeed::new(delivery, Vec::new());
    feed.publish(1, DebugFeedStage::Loading, vec![item("a"), item("b")]);
    next_control(&mut commands).await;
    feed.select("b").expect("selection");
    next_control(&mut commands).await;

    feed.publish(2, DebugFeedStage::Settled, vec![item("b"), item("a")]);
    let DeliveryCommand::Focus(retained) = next_control(&mut commands).await else {
        panic!("expected focus");
    };
    assert_eq!(retained.current_index, 0);

    feed.publish(3, DebugFeedStage::Settled, Vec::new());
    let DeliveryCommand::Focus(empty) = next_control(&mut commands).await else {
        panic!("expected focus");
    };
    assert!(empty.items.is_empty());
    assert_eq!(feed.snapshot().current_id, None);
}
