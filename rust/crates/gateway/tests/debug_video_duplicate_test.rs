#![cfg(feature = "video-debug-web")]

use ghostr_delivery::delivery_events::{command_channel, DeliveryCommand};
use ghostr_gateway::debug_videos::{DebugVideoRegistration, DebugVideos};

#[tokio::test]
async fn registering_the_same_url_updates_one_focus_item() {
    let (delivery, mut commands) = command_channel();
    let videos = DebugVideos::new(delivery);
    let first = DebugVideoRegistration {
        url: "https://cdn.example/video.mp4".to_owned(),
        size_bytes: None,
        duration_ms: None,
    };
    let mut updated = first.clone();
    updated.duration_ms = Some(42_000);

    videos.add(first).expect("first video");
    videos.add(updated).expect("updated video");

    let DeliveryCommand::Candidate(first) = commands.recv().await.expect("first candidate") else {
        panic!("expected candidate");
    };
    let _ = commands.recv().await.expect("first priority");
    let DeliveryCommand::Candidate(updated) = commands.recv().await.expect("updated candidate")
    else {
        panic!("expected candidate");
    };
    assert_eq!(updated.post, first.post);
    assert_eq!(updated.meta.duration_ms, Some(42_000));
}
