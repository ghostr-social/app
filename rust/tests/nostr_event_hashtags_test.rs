use nostr_sdk::{EventBuilder, Keys, Kind, Tag};
use rust_lib_ghostr::video::event_identity::canonical_video_events;

#[test]
fn normalizes_and_deduplicates_event_hashtags() {
    let keys = Keys::generate();
    let hash = "a".repeat(64);
    let event = EventBuilder::new(Kind::Custom(22), "Relay dance")
        .tags([
            Tag::parse(["t", " Dance "]).expect("hashtag tag"),
            Tag::parse(["t", "#dance"]).expect("prefixed hashtag tag"),
            Tag::parse(["t", "FOOTWORK"]).expect("uppercase hashtag tag"),
            Tag::parse(["t", "  "]).expect("blank hashtag tag"),
            Tag::parse([
                "imeta".to_owned(),
                "url https://media.example/video.mp4".to_owned(),
                format!("x {hash}"),
                "m video/mp4".to_owned(),
            ])
            .expect("video tag"),
        ])
        .sign_with_keys(&keys)
        .expect("signed event");

    let identities = canonical_video_events(&event);

    assert_eq!(identities.len(), 1);
    assert_eq!(identities[0].1.hashtags, vec!["dance", "footwork"]);
}
