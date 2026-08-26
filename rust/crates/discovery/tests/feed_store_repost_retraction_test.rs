use crate::content::deletions::deletion_claims;
use crate::content::reposts::feed_post_from_event;
use crate::feed::spec::FeedSpec;
use crate::feed::store::FeedStore;
use crate::tests::feed_support::{
    empty_graph, parsed, repost, signed_event, video_note, SignedEventFixture,
};
use nostr_sdk::{Keys, Kind};

#[test]
fn deleting_selected_wrapper_reveals_the_direct_occurrence() {
    let creator = Keys::generate();
    let reposter = Keys::generate();
    let original = video_note(&creator, "clip", 10);
    let wrapper = repost(&reposter, &original, 30);
    let deletion = signed_event(SignedEventFixture {
        keys: &reposter,
        kind: Kind::EventDeletion,
        content: "undo repost",
        tags: vec![vec!["e".to_owned(), wrapper.id.to_hex()]],
        created_at: 40,
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

    let post = &store.posts(feed)[0];
    assert!(post.repost.is_none());
    assert_eq!(post.feed_sort_at, 10);
}
