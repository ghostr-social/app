//! Protected video events remain repostable without exposing embeddable JSON.

mod feed_support;

use feed_support::{signed_event, SignedEventFixture};
use ghostr_discovery::content::parsing::video_post_from_event;
use nostr_sdk::{Keys, Kind};

#[test]
fn protected_video_preserves_its_flag_and_omits_signed_json() {
    let keys = Keys::generate();
    let event = signed_event(SignedEventFixture {
        keys: &keys,
        kind: Kind::TextNote,
        content: "https://cdn.example/protected.mp4",
        tags: vec![vec!["-".to_owned()]],
        created_at: 20,
    });

    let post = video_post_from_event(&event).expect("protected video parses");

    assert!(post.is_protected);
    assert_eq!(post.signed_event_json, None);
}
