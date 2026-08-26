use crate::content::reposts::feed_post_from_event;
use crate::tests::feed_support::{repost, signed_event, SignedEventFixture};
use nostr_sdk::{Keys, Kind};

#[test]
fn oversized_embedded_original_is_deferred_before_repost_admission() {
    let content = format!("https://cdn.example/clip.mp4 {}", "x".repeat(40 * 1024));
    let original = signed_event(SignedEventFixture {
        keys: &Keys::generate(),
        kind: Kind::TextNote,
        content: &content,
        tags: Vec::new(),
        created_at: 10,
    });
    let wrapper = repost(&Keys::generate(), &original, 20);

    assert!(feed_post_from_event(&wrapper).is_none());
}
