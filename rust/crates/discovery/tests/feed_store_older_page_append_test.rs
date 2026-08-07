//! An older page appends below the posts the viewer already scrolls —
//! never reordering them — and posts already present are skipped by
//! same-video identity, mirroring `FeedPagination.appendNew` (keyed by
//! VideoInteractionTarget) in
//! lib/features/video_catalog/presentation/feed_pagination.dart.

mod feed_support;

use feed_support::{addressable_video, parsed, parsed_posts, video_note};
use nostr_sdk::Keys;
use ghostr_discovery::feed_spec::FeedSpec;
use ghostr_discovery::feed_store::{FeedId, FeedStore};
use ghostr_discovery::social_graph::SocialGraph;

fn open_main(store: &mut FeedStore, viewer: &Keys) -> FeedId {
    store.open_feed(FeedSpec::MainFeed {
        viewer: Some(viewer.public_key()),
    })
}

fn slugs(store: &FeedStore, feed: FeedId) -> Vec<String> {
    store
        .posts(feed)
        .iter()
        .map(|post| post.meta.urls[0].clone())
        .collect()
}

#[test]
fn feed_store_appends_older_posts_below_the_current_list() {
    let keys = Keys::generate();
    let graph = SocialGraph::new(keys.public_key());
    let mut store = FeedStore::new();
    let feed = open_main(&mut store, &keys);
    store.ingest_first_page(
        feed,
        parsed_posts(&[video_note(&keys, "first", 50)]),
        &graph,
    );
    store.begin_load_more(feed);

    store.ingest_older_page(
        feed,
        parsed_posts(&[video_note(&keys, "older", 40)]),
        &graph,
    );

    assert_eq!(
        slugs(&store, feed),
        [
            "https://cdn.example/first.mp4".to_owned(),
            "https://cdn.example/older.mp4".to_owned(),
        ],
    );
}

#[test]
fn feed_store_skips_posts_already_present_by_same_video_identity() {
    let keys = Keys::generate();
    let graph = SocialGraph::new(keys.public_key());
    let mut store = FeedStore::new();
    let feed = open_main(&mut store, &keys);
    let seen = video_note(&keys, "seen", 50);
    let revision = addressable_video(&keys, "vid-1", "cut-one", 45);
    store.ingest_first_page(feed, parsed_posts(&[seen.clone(), revision]), &graph);
    store.begin_load_more(feed);

    // The same note again plus another revision of the same addressable
    // video: both resolve to identities already on screen.
    let older = vec![
        parsed(&seen),
        parsed(&addressable_video(&keys, "vid-1", "cut-two", 30)),
        parsed(&video_note(&keys, "fresh", 20)),
    ];
    store.ingest_older_page(feed, older, &graph);

    assert_eq!(
        slugs(&store, feed),
        [
            "https://cdn.example/seen.mp4".to_owned(),
            "https://cdn.example/cut-one.mp4".to_owned(),
            "https://cdn.example/fresh.mp4".to_owned(),
        ],
    );
}
