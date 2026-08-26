use crate::content::reposts::feed_post_from_event;
use crate::tests::feed_support::{signed_event, video_note, SignedEventFixture};
use nostr_sdk::{JsonUtil as _, Keys, Kind};

#[test]
fn wrapper_target_tags_must_match_the_verified_embedded_event() {
    let original = video_note(&Keys::generate(), "original", 10);
    let wrapper_keys = Keys::generate();
    let wrapper = signed_event(SignedEventFixture {
        keys: &wrapper_keys,
        kind: Kind::Custom(6),
        content: &original.as_json(),
        tags: vec![
            vec![
                "e".to_owned(),
                "11".repeat(32),
                "wss://relay.example".to_owned(),
            ],
            vec!["p".to_owned(), original.pubkey.to_hex()],
        ],
        created_at: 20,
    });

    assert!(feed_post_from_event(&wrapper).is_none());
}
