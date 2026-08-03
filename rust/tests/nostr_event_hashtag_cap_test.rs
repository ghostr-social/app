use nostr_sdk::{EventBuilder, Keys, Kind, Tag};
use rust_lib_ghostr::video::event_identity::{
    canonical_video_events, MAX_NATIVE_HASHTAGS_PER_EVENT,
};

#[test]
fn caps_the_hashtags_retained_per_event() {
    let keys = Keys::generate();
    let hash = "a".repeat(64);
    let mut tags: Vec<Tag> = (0..MAX_NATIVE_HASHTAGS_PER_EVENT + 5)
        .map(|index| Tag::parse(["t", &format!("tag{index}")]).expect("hashtag tag"))
        .collect();
    tags.push(
        Tag::parse([
            "imeta".to_owned(),
            "url https://media.example/video.mp4".to_owned(),
            format!("x {hash}"),
            "m video/mp4".to_owned(),
        ])
        .expect("video tag"),
    );
    let event = EventBuilder::new(Kind::Custom(22), "Relay dance")
        .tags(tags)
        .sign_with_keys(&keys)
        .expect("signed event");

    let identities = canonical_video_events(&event);

    assert_eq!(identities.len(), 1);
    assert_eq!(
        identities[0].1.hashtags.len(),
        MAX_NATIVE_HASHTAGS_PER_EVENT
    );
    assert_eq!(identities[0].1.hashtags[0], "tag0");
}
