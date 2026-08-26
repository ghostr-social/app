use crate::content::deletions::deletion_claims;
use crate::content::reposts::feed_post_from_event;
use crate::feed::spec::FeedSpec;
use crate::feed::store::FeedStore;
use crate::tests::feed_support::SignedEventFixture;
use crate::tests::feed_support::{addressable_video, empty_graph, parsed, repost, signed_event};
use nostr_sdk::{Keys, Kind};

#[test]
fn coordinate_repost_revives_with_a_newer_original_revision() {
    let creator = Keys::generate();
    let reposter = Keys::generate();
    let old = addressable_video(&creator, "clip", "old", 10);
    let current = addressable_video(&creator, "clip", "current", 30);
    let wrapper = repost(&reposter, &old, 40);
    let coordinate = format!("34235:{}:clip", creator.public_key());
    let deletion = signed_event(SignedEventFixture {
        keys: &creator,
        kind: Kind::EventDeletion,
        content: "delete old revisions",
        tags: vec![vec!["a".to_owned(), coordinate]],
        created_at: 20,
    });
    let mut store = FeedStore::new();
    let feed = store.open_feed(FeedSpec::Following {
        viewer: None,
        follows: vec![creator.public_key(), reposter.public_key()],
    });
    store.ingest_first_page(
        feed,
        vec![feed_post_from_event(&wrapper).expect("valid test fixture")],
        &empty_graph(),
    );

    store.ingest_deletions(feed, deletion_claims(&[deletion]), &empty_graph());
    assert!(store.posts(feed).is_empty());
    store.ingest_older_page(feed, vec![parsed(&current)], &empty_graph());

    let post = &store.posts(feed)[0];
    assert_eq!(post.event_id, current.id.to_hex());
    assert_eq!(
        post.repost.as_ref().expect("valid test fixture").event_id,
        wrapper.id.to_hex()
    );
    assert_eq!(post.feed_sort_at, 40);
}
