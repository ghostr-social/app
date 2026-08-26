use crate::content::deletions::deletion_claims;
use crate::content::reposts::feed_post_from_event;
use crate::feed::spec::FeedSpec;
use crate::feed::store::FeedStore;
use crate::tests::feed_support::{
    empty_graph, parsed, repost, signed_event, video_note, SignedEventFixture,
};
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
        vec![
            parsed(&original),
            feed_post_from_event(&wrapper).expect("valid test fixture"),
        ],
        &empty_graph(),
    );

    store.ingest_deletions(feed, deletion_claims(&[deletion]), &empty_graph());

    assert!(store.posts(feed).is_empty());
}
