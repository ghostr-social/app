use nostr_sdk::{EventBuilder, Keys, Kind, Tag};
use rust_lib_ghostr::video::event_identity::canonical_native_videos;

#[test]
fn rejects_a_video_event_with_an_invalid_sha256_digest() {
    let event = EventBuilder::new(Kind::Custom(22), "Invalid digest")
        .tags([Tag::parse([
            "imeta",
            "url https://media.example/video.mp4",
            "x abc123",
            "m video/mp4",
        ])
        .expect("video tag")])
        .sign_with_keys(&Keys::generate())
        .expect("signed event");

    assert!(canonical_native_videos(&event).is_empty());
}
