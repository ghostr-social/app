//! An empty older page exhausts a canonical feed (`_nextCursor` returns
//! null in lib/features/video_catalog/domain/filtered_video_feed_repository.dart)
//! but a query feed keeps its cursor and keeps hunting — the viewer asked
//! for exactly this content, so the search never reports itself finished
//! (lib/features/video_catalog/domain/query_video_feed_repository.dart
//! `_freshMatches`).

mod feed_support;

use feed_support::{parsed_posts, video_note};
use nostr_sdk::{Keys, Timestamp};
use rust_lib_ghostr::discovery::feed_spec::FeedSpec;
use rust_lib_ghostr::discovery::feed_store::FeedStore;
use rust_lib_ghostr::discovery::social_graph::SocialGraph;

#[test]
fn feed_store_main_feed_exhausts_on_an_empty_older_page() {
    let keys = Keys::generate();
    let graph = SocialGraph::new(keys.public_key());
    let mut store = FeedStore::new();
    let feed = store.open_feed(FeedSpec::MainFeed {
        viewer: keys.public_key(),
    });
    store.ingest_first_page(feed, parsed_posts(&[video_note(&keys, "only", 50)]), &graph);
    store.begin_load_more(feed);

    store.ingest_older_page(feed, Vec::new(), &graph);

    assert_eq!(store.begin_load_more(feed), None);
}

#[test]
fn feed_store_search_feed_keeps_its_cursor_on_an_empty_older_page() {
    let keys = Keys::generate();
    let graph = SocialGraph::new(keys.public_key());
    let mut store = FeedStore::new();
    let feed = store.open_feed(FeedSpec::Search("surf".to_owned()));
    store.ingest_first_page(feed, parsed_posts(&[video_note(&keys, "surf", 50)]), &graph);
    store.begin_load_more(feed);

    store.ingest_older_page(feed, Vec::new(), &graph);

    // The next swipe digs again from the same spot.
    assert_eq!(store.begin_load_more(feed), Some(Timestamp::from(49)));
}

#[test]
fn feed_store_query_feed_with_an_empty_first_load_has_no_cursor() {
    let keys = Keys::generate();
    let graph = SocialGraph::new(keys.public_key());
    let mut store = FeedStore::new();
    let feed = store.open_feed(FeedSpec::Hashtag("#cats".to_owned()));
    store.ingest_first_page(feed, Vec::new(), &graph);

    // An empty first load leaves a query feed with no cursor at all —
    // the head re-query (`loadFeed`) is how it hunts, not pagination.
    assert_eq!(store.begin_load_more(feed), None);
}
