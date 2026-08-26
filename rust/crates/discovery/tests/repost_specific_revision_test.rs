use crate::content::reposts::feed_post_from_event;
use crate::tests::feed_support::{addressable_video, signed_event, SignedEventFixture};
use nostr_sdk::{JsonUtil as _, Keys, Kind};

#[test]
fn embedded_generic_repost_can_target_one_specific_revision() {
    let original = addressable_video(&Keys::generate(), "clip", "video", 10);
    let wrapper = signed_event(SignedEventFixture {
        keys: &Keys::generate(),
        kind: Kind::Custom(16),
        content: &original.as_json(),
        tags: Vec::new(),
        created_at: 20,
    });

    assert!(feed_post_from_event(&wrapper).is_some());
}
