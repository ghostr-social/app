//! A feed now lives as long as the app does
//! (`lib/features/video_catalog/data/rust_feed_sessions.dart`), so the
//! store bounds what one canonical feed keeps: the newest rows stay,
//! the tail the viewer already scrolled past is dropped, and trimming
//! it never rewinds pagination.

use crate::content::social_graph::SocialGraph;
use crate::feed::spec::FeedSpec;
use crate::feed::store::{FeedId, FeedStore, FEED_POST_RETENTION};
use crate::tests::feed_store_support::page;
use nostr_sdk::{Keys, Timestamp};

const PAGE: u64 = 100;
const NEWEST: u64 = 10_000;

/// A feed fed one first page and older pages until it holds more rows
/// than it may keep.
fn overfilled_feed() -> (FeedStore, FeedId, SocialGraph) {
    let graph = SocialGraph::new(Keys::generate().public_key());
    let mut store = FeedStore::new();
    let feed = store.open_feed(FeedSpec::MainFeed { viewer: None });
    store.ingest_first_page(feed, page(NEWEST, PAGE), &graph);
    let pages = FEED_POST_RETENTION as u64 / PAGE + 1;
    for index in 1..pages {
        let newest = NEWEST - index * PAGE;
        store.begin_load_more_at(feed, None);
        store.ingest_older_page(feed, page(newest, PAGE), &graph);
    }
    (store, feed, graph)
}

#[test]
fn a_long_lived_feed_keeps_only_the_newest_retained_rows() {
    let (store, feed, _graph) = overfilled_feed();

    let posts = store.posts(feed);
    assert_eq!(posts.len(), FEED_POST_RETENTION);
    assert_eq!(posts[0].created_at, NEWEST);
    let oldest_kept = NEWEST + 1 - FEED_POST_RETENTION as u64;
    assert_eq!(posts[posts.len() - 1].created_at, oldest_kept);
}

#[test]
fn trimming_the_tail_leaves_pagination_where_the_last_page_ended() {
    let (mut store, feed, _graph) = overfilled_feed();

    let fetched_pages = FEED_POST_RETENTION as u64 / PAGE + 1;
    let oldest_fetched = NEWEST + 1 - fetched_pages * PAGE;
    assert_eq!(
        store.begin_load_more_at(feed, None),
        Some(Timestamp::from(oldest_fetched - 1)),
        "the cursor follows what was fetched, not what was kept"
    );
}
