use crate::content::reposts::feed_post_from_event;
use crate::feed::assembly::test_support::canonical_posts;
use crate::tests::feed_support::{addressable_video, parsed, signed_event, SignedEventFixture};
use nostr_sdk::{JsonUtil as _, Keys, Kind};

#[test]
fn specific_repost_of_stale_revision_keeps_its_embedded_media() {
    let creator = Keys::generate();
    let stale = addressable_video(&creator, "clip", "stale", 10);
    let current = addressable_video(&creator, "clip", "current", 20);
    let wrapper = signed_event(SignedEventFixture {
        keys: &Keys::generate(),
        kind: Kind::Custom(16),
        content: &stale.as_json(),
        tags: vec![vec!["e".to_owned(), stale.id.to_hex()]],
        created_at: 30,
    });
    let occurrences = vec![
        parsed(&current),
        feed_post_from_event(&wrapper).expect("specific repost parses"),
    ];

    let posts = canonical_posts(occurrences);

    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].event_id, stale.id.to_hex());
    assert_eq!(posts[0].meta.urls, ["https://cdn.example/stale.mp4"]);
    assert_eq!(posts[0].feed_sort_at, 30);
}
