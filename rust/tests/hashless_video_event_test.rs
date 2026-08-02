use nostr_sdk::{EventBuilder, Keys, Kind, Tag};
use rust_lib_ghostr::video::event_identity::canonical_native_videos;

#[test]
fn derives_a_stable_cache_key_when_nip92_has_no_digest() {
    let event = EventBuilder::new(Kind::Custom(22), "Hashless relay video")
        .tags([Tag::parse([
            "imeta",
            "url https://media.example/video.mp4",
            "m video/mp4",
        ])
        .expect("video tag")])
        .sign_with_keys(&Keys::generate())
        .expect("signed event");

    let first = canonical_native_videos(&event);
    let second = canonical_native_videos(&event);

    assert_eq!(first.len(), 1);
    assert_eq!(first[0].video.id.len(), 64);
    assert_eq!(first[0].video.id, second[0].video.id);
}
