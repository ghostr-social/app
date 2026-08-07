#![cfg(feature = "video-debug-web")]

use rust_lib_ghostr::video::debug_videos::{DebugVideoRegistration, DebugVideos};
use rust_lib_ghostr::video::delivery_events::command_channel;

#[test]
fn debug_video_registration_rejects_invalid_urls_and_measurements() {
    let (delivery, _) = command_channel();
    let videos = DebugVideos::new(delivery);
    let oversized = format!("https://example.com/{}", "x".repeat(8_193));
    let cases = [
        registration(""),
        registration("not a url"),
        registration("file:///tmp/video.mp4"),
        registration(&oversized),
        with_size(registration("https://example.com/video.mp4"), 0),
        with_duration(registration("https://example.com/video.mp4"), 0),
    ];

    for registration in cases {
        assert!(videos.add(registration).is_err());
    }
}

fn registration(url: &str) -> DebugVideoRegistration {
    DebugVideoRegistration {
        url: url.to_owned(),
        size_bytes: None,
        duration_ms: None,
    }
}

fn with_size(mut registration: DebugVideoRegistration, size: u64) -> DebugVideoRegistration {
    registration.size_bytes = Some(size);
    registration
}

fn with_duration(
    mut registration: DebugVideoRegistration,
    duration: u64,
) -> DebugVideoRegistration {
    registration.duration_ms = Some(duration);
    registration
}
