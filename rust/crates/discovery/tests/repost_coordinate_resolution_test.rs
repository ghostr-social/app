mod feed_support;

use feed_support::{addressable_video, empty_graph, parsed, repost};
use ghostr_discovery::content::reposts::feed_post_from_event;
use ghostr_discovery::feed::spec::FeedSpec;
use ghostr_discovery::feed::store::FeedStore;
use nostr_sdk::Keys;

#[test]
fn coordinate_repost_resolves_the_newest_verified_revision_in_its_batch() {
    let creator = Keys::generate();
    let stale = addressable_video(&creator, "clip", "stale", 10);
    let current = addressable_video(&creator, "clip", "current", 20);
    let reposter = Keys::generate();
    let wrapper = repost(&reposter, &stale, 30);
    let mut store = FeedStore::new();
    let feed = store.open_feed(FeedSpec::Following {
        viewer: None,
        follows: vec![reposter.public_key()],
    });

    store.ingest_first_page(
        feed,
        vec![feed_post_from_event(&wrapper).unwrap(), parsed(&current)],
        &empty_graph(),
    );
    let repost = &store.posts(feed)[0];

    assert_eq!(repost.event_id, current.id.to_hex());
    assert_eq!(repost.meta.urls, ["https://cdn.example/current.mp4"]);
    assert_eq!(repost.feed_sort_at, 30);
}
