#![cfg(feature = "video-debug-web")]

use ghostr_delivery::delivery_events::{command_channel, DeliveryCommand};
use ghostr_gateway::debug::videos::{DebugVideoRegistration, DebugVideos};

#[tokio::test]
async fn selecting_a_debug_video_replaces_delivery_focus() {
    let (delivery, mut commands) = command_channel();
    let videos = DebugVideos::new(delivery);
    let first = videos.add(registration("first")).expect("first video");
    let second = videos.add(registration("second")).expect("second video");

    assert!(videos.select(&second));

    let DeliveryCommand::Focus(focus) = commands.receivers().0.recv().await.expect("focus command")
    else {
        panic!("selection must update focus, not only candidate priority");
    };
    assert_eq!(focus.current_index, 1);
    assert_eq!(focus.items[0].post.as_str(), first);
    assert_eq!(focus.items[1].post.as_str(), second);
}

fn registration(name: &str) -> DebugVideoRegistration {
    DebugVideoRegistration {
        url: format!("https://media.example/{name}.mp4"),
        mirrors: Vec::new(),
        size_bytes: Some(1_000),
        duration_ms: Some(1_000),
    }
}
