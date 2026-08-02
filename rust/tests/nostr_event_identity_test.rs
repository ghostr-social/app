use nostr_sdk::{EventBuilder, Keys, Kind, Tag};
use rust_lib_ghostr::video::event_identity::canonical_video_events;

#[test]
fn retains_canonical_identity_for_each_video_hash() {
    let keys = Keys::generate();
    let hash = "a".repeat(64);
    let event = EventBuilder::new(Kind::Custom(34235), "Relay dance")
        .tags([
            Tag::parse(["d", "dance"]).expect("identifier tag"),
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
    assert_eq!(identities[0].0, hash);
    assert_eq!(identities[0].1.event_id, event.id.to_hex());
    assert_eq!(identities[0].1.author_public_key_hex, event.pubkey.to_hex());
    assert_eq!(identities[0].1.kind, 34235);
    assert_eq!(identities[0].1.identifier.as_deref(), Some("dance"));
    assert_eq!(identities[0].1.content, "Relay dance");
}
