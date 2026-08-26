use crate::feed::spec::FeedSpec;
use crate::feed::store::FeedStore;
use crate::tests::feed_support::{empty_graph, parsed, signed_event, SignedEventFixture};
use nostr_sdk::{Keys, Kind};

#[test]
fn hashtag_feed_does_not_adopt_an_untagged_coordinate_revision() {
    let creator = Keys::generate();
    let tagged = signed_event(SignedEventFixture {
        keys: &creator,
        kind: Kind::Custom(34235),
        content: "https://cdn.example/tagged.mp4",
        tags: vec![
            vec!["d".to_owned(), "clip".to_owned()],
            vec!["t".to_owned(), "cats".to_owned()],
        ],
        created_at: 10,
    });
    let untagged = signed_event(SignedEventFixture {
        keys: &creator,
        kind: Kind::Custom(34235),
        content: "https://cdn.example/untagged.mp4",
        tags: vec![vec!["d".to_owned(), "clip".to_owned()]],
        created_at: 20,
    });
    let mut store = FeedStore::new();
    let feed = store.open_feed(FeedSpec::Hashtag("cats".to_owned()));

    store.ingest_first_page(
        feed,
        vec![parsed(&tagged), parsed(&untagged)],
        &empty_graph(),
    );

    assert_eq!(
        store.posts(feed)[0].meta.urls,
        ["https://cdn.example/tagged.mp4"]
    );
}
