use crate::content::deletions::deletion_claims;
use crate::feed::spec::FeedSpec;
use crate::feed::store::FeedStore;
use crate::tests::feed_support::{
    addressable_video, empty_graph, parsed, signed_event, SignedEventFixture,
};
use nostr_sdk::{Keys, Kind};

#[test]
fn address_deletion_hides_only_revisions_published_by_its_cutoff() {
    let creator = Keys::generate();
    let old = addressable_video(&creator, "clip", "old", 10);
    let new = addressable_video(&creator, "clip", "new", 30);
    let coordinate = format!("34235:{}:clip", creator.public_key());
    let deletion = signed_event(SignedEventFixture {
        keys: &creator,
        kind: Kind::EventDeletion,
        content: "delete old revisions",
        tags: vec![vec!["a".to_owned(), coordinate]],
        created_at: 20,
    });
    let mut store = FeedStore::new();
    let feed = store.open_feed(FeedSpec::Profile(vec![creator.public_key()]));
    store.ingest_first_page(feed, vec![parsed(&old)], &empty_graph());

    store.ingest_deletions(feed, deletion_claims(&[deletion]), &empty_graph());
    assert!(store.posts(feed).is_empty());

    store.ingest_older_page(feed, vec![parsed(&new)], &empty_graph());
    assert_eq!(store.posts(feed)[0].event_id, new.id.to_hex());
}
