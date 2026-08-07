//! Older-page windows: a fresh load rebases the cursor one second below
//! the oldest *visible* post (`FeedPagination.restartFrom` called with the
//! policy-selected posts in feed_cubit.dart `_acceptLoad`), one older
//! request is in flight at a time (`beginLoad`), and a completed page
//! advances the cursor by what was *fetched*
//! (filtered_video_feed_repository.dart `_nextCursor`).

mod discovery_support;
mod feed_support;

use discovery_support::{mute_list, p_tag};
use feed_support::{parsed_posts, video_note};
use nostr_sdk::{Keys, Timestamp};
use ghostr_discovery::feed::spec::FeedSpec;
use ghostr_discovery::feed::store::FeedStore;
use ghostr_discovery::content::social_graph::SocialGraph;

fn main_feed(
    store: &mut FeedStore,
    viewer: &Keys,
) -> ghostr_discovery::feed::store::FeedId {
    store.open_feed(FeedSpec::MainFeed {
        viewer: Some(viewer.public_key()),
    })
}

#[test]
fn feed_store_rebases_the_cursor_below_the_oldest_visible_post() {
    let (session, muted, visible) = (Keys::generate(), Keys::generate(), Keys::generate());
    let mut graph = SocialGraph::new(session.public_key());
    graph.ingest(&mute_list(&session, vec![p_tag(&muted.public_key())], 5));
    let mut store = FeedStore::new();
    let feed = main_feed(&mut store, &session);

    // The oldest fetched post is muted; the cursor follows the visible one.
    let fetched = parsed_posts(&[
        video_note(&visible, "kept", 100),
        video_note(&muted, "hidden", 40),
    ]);
    store.ingest_first_page(feed, fetched, &graph);

    assert_eq!(store.begin_load_more(feed), Some(Timestamp::from(99)));
}

#[test]
fn feed_store_allows_one_older_request_in_flight() {
    let session = Keys::generate();
    let graph = SocialGraph::new(session.public_key());
    let mut store = FeedStore::new();
    let feed = main_feed(&mut store, &session);
    store.ingest_first_page(feed, parsed_posts(&[video_note(&session, "a", 50)]), &graph);

    assert_eq!(store.begin_load_more(feed), Some(Timestamp::from(49)));
    assert_eq!(store.begin_load_more(feed), None);
    store.fail_load_more(feed);
    assert_eq!(store.begin_load_more(feed), Some(Timestamp::from(49)));
}

#[test]
fn feed_store_advances_the_cursor_by_the_fetched_page() {
    let session = Keys::generate();
    let graph = SocialGraph::new(session.public_key());
    let mut store = FeedStore::new();
    let feed = main_feed(&mut store, &session);
    store.ingest_first_page(feed, parsed_posts(&[video_note(&session, "a", 50)]), &graph);
    store.begin_load_more(feed);

    let older = parsed_posts(&[video_note(&session, "b", 40), video_note(&session, "c", 30)]);
    store.ingest_older_page(feed, older, &graph);

    assert_eq!(store.begin_load_more(feed), Some(Timestamp::from(29)));
}
