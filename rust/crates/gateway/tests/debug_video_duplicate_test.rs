#![cfg(feature = "video-debug-web")]

use ghostr_delivery::delivery_events::{command_channel, DeliveryCommand};
use ghostr_gateway::debug::videos::{DebugVideoRegistration, DebugVideos};

#[tokio::test]
async fn registering_the_same_url_updates_one_focus_item() {
    let (delivery, mut commands) = command_channel();
    let videos = DebugVideos::new(delivery);
    let first = DebugVideoRegistration {
        url: "https://cdn.example/video.mp4".to_owned(),
        mirrors: Vec::new(),
        size_bytes: None,
        duration_ms: None,
    };
    let mut updated = first.clone();
    updated.duration_ms = Some(42_000);

    let id = videos.add(first).expect("first video");
    videos.add(updated).expect("updated video");
    assert!(videos.select(&id));

    let DeliveryCommand::Focus(focus) = commands.receivers().0.recv().await.expect("focus") else {
        panic!("expected focus");
    };
    assert_eq!(focus.items.len(), 1);
    assert_eq!(focus.items[0].post.as_str(), id);
    assert_eq!(focus.items[0].meta.duration_ms, Some(42_000));
}
