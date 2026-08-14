mod feed_support;

use feed_support::{empty_graph, parsed, repost, signed_event, video_note, SignedEventFixture};
use ghostr_discovery::content::deletions::deletion_claims;
use ghostr_discovery::content::reposts::feed_post_from_event;
use ghostr_discovery::feed::spec::FeedSpec;
use ghostr_discovery::feed::store::FeedStore;
use nostr_sdk::{Keys, Kind};

#[test]
fn original_event_deletion_hides_direct_and_reposted_occurrences() {
    let creator = Keys::generate();
    let reposter = Keys::generate();
    let original = video_note(&creator, "clip", 10);
    let wrapper = repost(&reposter, &original, 30);
    let deletion = signed_event(SignedEventFixture {
        keys: &creator,
        kind: Kind::EventDeletion,
        content: "deleted original",
        tags: vec![vec!["e".to_owned(), original.id.to_hex()]],
        created_at: 5,
    });
    let mut store = FeedStore::new();
    let feed = store.open_feed(FeedSpec::Following {
        viewer: None,
        follows: vec![creator.public_key(), reposter.public_key()],
    });
    store.ingest_first_page(
        feed,
        vec![parsed(&original), feed_post_from_event(&wrapper).unwrap()],
        &empty_graph(),
    );

    store.ingest_deletions(feed, deletion_claims(&[deletion]), &empty_graph());

    assert!(store.posts(feed).is_empty());
}
