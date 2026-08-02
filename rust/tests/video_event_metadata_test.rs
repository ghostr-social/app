use nostr_sdk::{EventBuilder, Keys, Kind, Tag};
use rust_lib_ghostr::video::event_identity::canonical_native_videos;

#[test]
fn maps_validated_nip71_metadata_into_a_native_video() {
    let digest = "B".repeat(64);
    let event = EventBuilder::new(Kind::Custom(34236), "Event fallback")
        .tags([
            Tag::parse(["d", "portrait"]).expect("identifier"),
            Tag::parse(["title", "Relay anthem"]).expect("song title"),
            Tag::parse([
                "imeta".to_owned(),
                "url http://media.example/video.mp4".to_owned(),
                format!("x {digest}"),
                "m video/webm".to_owned(),
                "title Portrait clip".to_owned(),
            ])
            .expect("video metadata"),
        ])
        .sign_with_keys(&Keys::generate())
        .expect("signed event");

    let videos = canonical_native_videos(&event);

    assert_eq!(videos.len(), 1);
    assert_eq!(videos[0].video.id, "b".repeat(64));
    assert_eq!(videos[0].video.title, "Portrait clip");
    assert_eq!(videos[0].video.song_name, "Relay anthem");
    assert!(videos[0].video.user.npub.is_some());
    assert_eq!(videos[0].identity.identifier.as_deref(), Some("portrait"));
}
