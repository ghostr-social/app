mod feed_support;

use feed_support::{signed_event, video_note, SignedEventFixture};
use ghostr_discovery::content::reposts::feed_post_from_event;
use nostr_sdk::{JsonUtil, Keys, Kind};

#[test]
fn kind_six_accepts_a_missing_recommended_p_tag() {
    let original = video_note(&Keys::generate(), "clip", 10);
    let wrapper = signed_event(SignedEventFixture {
        keys: &Keys::generate(),
        kind: Kind::Custom(6),
        content: &original.as_json(),
        tags: vec![vec![
            "e".to_owned(),
            original.id.to_hex(),
            "wss://relay.example".to_owned(),
        ]],
        created_at: 20,
    });

    assert!(feed_post_from_event(&wrapper).is_some());
}
