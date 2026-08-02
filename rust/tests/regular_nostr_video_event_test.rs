use nostr_sdk::{EventBuilder, Keys, Kind, Tag};
use rust_lib_ghostr::video::event_identity::canonical_native_videos;

#[test]
fn discovers_a_regular_nip71_video_with_canonical_identity() {
    let keys = Keys::generate();
    let hash = "a".repeat(64);
    let event = EventBuilder::new(Kind::Custom(22), "A short relay video")
        .tags([
            Tag::parse(["d", "ignored-on-regular-events"]).expect("d tag"),
            Tag::parse([
                "imeta".to_owned(),
                "url https://media.example/short.mp4".to_owned(),
                format!("x {hash}"),
                "m video/mp4".to_owned(),
            ])
            .expect("video tag"),
        ])
        .sign_with_keys(&keys)
        .expect("signed event");

    let videos = canonical_native_videos(&event);

    assert_eq!(videos.len(), 1);
    assert_eq!(videos[0].video.id, hash);
    assert_eq!(videos[0].video.url, "https://media.example/short.mp4");
    assert_eq!(videos[0].identity.event_id, event.id.to_hex());
    assert_eq!(videos[0].identity.kind, 22);
    assert!(videos[0].identity.identifier.is_none());
}
