//! Feed pages hide muted creators' posts: the main feed drops them like
//! the blocked filter in
//! lib/features/video_catalog/domain/video_feed_policy.dart, and search
//! feeds drop them like `_selectPosts` in
//! lib/features/video_catalog/domain/discovery_video_search_repository.dart.

mod discovery_support;
mod feed_support;

use discovery_support::{mute_list, p_tag};
use feed_support::{parsed_posts, video_note};
use nostr_sdk::Keys;
use rust_lib_ghostr::discovery::feed_spec::FeedSpec;
use rust_lib_ghostr::discovery::feed_store::FeedStore;
use rust_lib_ghostr::discovery::social_graph::SocialGraph;

fn muting_graph(session: &Keys, muted: &Keys) -> SocialGraph {
    let mut graph = SocialGraph::new(session.public_key());
    graph.ingest(&mute_list(session, vec![p_tag(&muted.public_key())], 5));
    graph
}

#[test]
fn feed_store_main_feed_drops_muted_creators_posts() {
    let (session, muted, visible) = (Keys::generate(), Keys::generate(), Keys::generate());
    let graph = muting_graph(&session, &muted);
    let mut store = FeedStore::new();
    let feed = store.open_feed(FeedSpec::MainFeed {
        viewer: session.public_key(),
    });

    let fetched = parsed_posts(&[
        video_note(&muted, "spam", 30),
        video_note(&visible, "keeper", 20),
    ]);
    store.ingest_first_page(feed, fetched, &graph);

    let authors: Vec<&str> = store
        .posts(feed)
        .iter()
        .map(|post| post.author_pubkey.as_str())
        .collect();
    assert_eq!(authors, [visible.public_key().to_hex().as_str()]);
}

#[test]
fn feed_store_search_feed_drops_muted_creators_posts() {
    let (session, muted, visible) = (Keys::generate(), Keys::generate(), Keys::generate());
    let graph = muting_graph(&session, &muted);
    let mut store = FeedStore::new();
    let feed = store.open_feed(FeedSpec::Search("surf".to_owned()));

    let fetched = parsed_posts(&[
        video_note(&muted, "surf-spam", 30),
        video_note(&visible, "surf-keeper", 20),
    ]);
    store.ingest_first_page(feed, fetched, &graph);

    let authors: Vec<&str> = store
        .posts(feed)
        .iter()
        .map(|post| post.author_pubkey.as_str())
        .collect();
    assert_eq!(authors, [visible.public_key().to_hex().as_str()]);
}
