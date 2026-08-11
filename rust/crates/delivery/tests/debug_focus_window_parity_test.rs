#![cfg(feature = "video-debug-web")]

use ghostr_delivery::debug::feed::{DebugFeed, DebugFeedItem, DebugFeedStage};
use ghostr_delivery::delivery_events::{command_channel, DeliveryCommand};
use ghostr_engine::{DeliveryKind, VideoMeta};

#[tokio::test]
async fn debug_focus_matches_the_product_two_behind_six_ahead_window() {
    let (delivery, mut commands) = command_channel();
    let feed = DebugFeed::new(delivery, Vec::new());
    let items: Vec<_> = (0..12).map(item).collect();
    feed.publish(1, DebugFeedStage::Settled, items);
    commands.receivers().0.recv().await.expect("initial focus");

    feed.select("post-5").expect("feed selection");
    let DeliveryCommand::Focus(focus) = commands.receivers().0.recv().await.expect("focus") else {
        panic!("expected focus command");
    };
    let ids: Vec<_> = focus.items.iter().map(|item| item.post.as_str()).collect();

    assert_eq!(
        ids,
        (3..12)
            .map(|index| format!("post-{index}"))
            .collect::<Vec<_>>()
    );
    assert_eq!(focus.current_index, 2);
}

fn item(index: usize) -> DebugFeedItem {
    let id = format!("post-{index}");
    DebugFeedItem {
        event_id: format!("event-{index}"),
        title: None,
        creator: "creator".to_owned(),
        created_at: 100 - index as u64,
        meta: VideoMeta {
            urls: vec![format!("https://media.example/{id}.mp4")],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(1_000),
            duration_ms: Some(1_000),
        },
        id,
    }
}
