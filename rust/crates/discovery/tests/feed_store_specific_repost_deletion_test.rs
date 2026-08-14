mod feed_support;

use feed_support::{addressable_video, empty_graph, parsed, signed_event, SignedEventFixture};
use ghostr_discovery::content::deletions::deletion_claims;
use ghostr_discovery::content::reposts::feed_post_from_event;
use ghostr_discovery::feed::spec::FeedSpec;
use ghostr_discovery::feed::store::FeedStore;
use nostr_sdk::{JsonUtil, Keys, Kind};

#[test]
fn deleted_specific_repost_does_not_adopt_a_newer_revision() {
    let creator = Keys::generate();
    let reposter = Keys::generate();
    let old = addressable_video(&creator, "clip", "old", 10);
    let current = addressable_video(&creator, "clip", "current", 30);
    let wrapper = signed_event(SignedEventFixture {
        keys: &reposter,
        kind: Kind::Custom(16),
        content: &old.as_json(),
        tags: vec![vec!["e".to_owned(), old.id.to_hex()]],
        created_at: 40,
    });
    let deletion = signed_event(SignedEventFixture {
        keys: &creator,
        kind: Kind::EventDeletion,
        content: "delete original",
        tags: vec![vec!["e".to_owned(), old.id.to_hex()]],
        created_at: 20,
    });
    let mut store = FeedStore::new();
    let feed = store.open_feed(FeedSpec::Following {
        viewer: None,
        follows: vec![creator.public_key(), reposter.public_key()],
    });
    store.ingest_first_page(
        feed,
        vec![feed_post_from_event(&wrapper).unwrap()],
        &empty_graph(),
    );
    store.ingest_deletions(feed, deletion_claims(&[deletion]), &empty_graph());
    store.ingest_older_page(feed, vec![parsed(&current)], &empty_graph());

    let post = &store.posts(feed)[0];
    assert_eq!(post.event_id, current.id.to_hex());
    assert!(post.repost.is_none());
    assert_eq!(post.feed_sort_at, 30);
}
