use nostr_sdk::{EventBuilder, Keys, Kind, Tag};
use rust_lib_ghostr::video::event_identity::canonical_native_videos;

#[test]
fn rejects_a_video_event_without_a_video_mime_type() {
    let hash = "a".repeat(64);
    let event = EventBuilder::new(Kind::Custom(22), "Missing MIME")
        .tags([Tag::parse([
            "imeta".to_owned(),
            "url https://media.example/video.mp4".to_owned(),
            format!("x {hash}"),
        ])
        .expect("video tag")])
        .sign_with_keys(&Keys::generate())
        .expect("signed event");

    assert!(canonical_native_videos(&event).is_empty());
}
