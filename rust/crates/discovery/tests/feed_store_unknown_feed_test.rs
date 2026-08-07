//! Pages for a closed or unknown feed ID never recreate that feed.

mod feed_support;

use feed_support::empty_graph;
use ghostr_discovery::feed_store::{FeedId, FeedStore};

#[test]
fn unknown_feed_ignores_fresh_and_older_pages() {
    let graph = empty_graph();
    let unknown = FeedId(u64::MAX);
    let mut store = FeedStore::new();

    store.ingest_first_page(unknown, Vec::new(), &graph);
    let appended = store.ingest_older_page(unknown, Vec::new(), &graph);

    assert!(store.posts(unknown).is_empty());
    assert!(!appended);
}
