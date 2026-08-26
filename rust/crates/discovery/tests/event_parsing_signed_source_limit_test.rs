use crate::content::parsing::video_post_from_event;
use crate::tests::feed_support::{signed_event, SignedEventFixture};
use nostr_sdk::{Keys, Kind};

#[test]
fn oversized_video_stays_playable_without_retaining_its_signed_source() {
    let content = format!("https://cdn.example/clip.mp4 {}", "x".repeat(40 * 1024));
    let event = signed_event(SignedEventFixture {
        keys: &Keys::generate(),
        kind: Kind::TextNote,
        content: &content,
        tags: Vec::new(),
        created_at: 10,
    });

    let post = video_post_from_event(&event).expect("large video still parses");

    assert_eq!(post.meta.urls, ["https://cdn.example/clip.mp4"]);
    assert!(post.signed_event_json.is_none());
}
