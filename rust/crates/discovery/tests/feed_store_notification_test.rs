//! Subscribers hear about a feed exactly when its visible list changes:
//! every fresh load notifies (the cubit re-emits on every load), while an
//! older page that adds nothing new stays silent — mirrors the early
//! return in feed_cubit.dart `_appendPage` ("A page that adds nothing new
//! stops the digging chain").

mod feed_support;

use feed_support::{parsed, parsed_posts, video_note};
use nostr_sdk::Keys;
use ghostr_discovery::feed::spec::FeedSpec;
use ghostr_discovery::feed::store::FeedStore;
use ghostr_discovery::content::social_graph::SocialGraph;

#[test]
fn feed_store_notifies_subscribers_on_a_fresh_load() {
    let keys = Keys::generate();
    let graph = SocialGraph::new(keys.public_key());
    let mut store = FeedStore::new();
    let feed = store.open_feed(FeedSpec::MainFeed {
        viewer: Some(keys.public_key()),
    });
    let mut updates = store.subscribe(feed).expect("open feed subscribes");
    updates.borrow_and_update();

    store.ingest_first_page(feed, parsed_posts(&[video_note(&keys, "a", 50)]), &graph);

    assert!(updates.has_changed().expect("feed still open"));
}

#[test]
fn feed_store_stays_silent_when_an_older_page_adds_nothing() {
    let keys = Keys::generate();
    let graph = SocialGraph::new(keys.public_key());
    let mut store = FeedStore::new();
    let feed = store.open_feed(FeedSpec::MainFeed {
        viewer: Some(keys.public_key()),
    });
    let seen = video_note(&keys, "seen", 50);
    store.ingest_first_page(feed, vec![parsed(&seen)], &graph);
    let mut updates = store.subscribe(feed).expect("open feed subscribes");
    updates.borrow_and_update();

    store.begin_load_more(feed);
    store.ingest_older_page(feed, vec![parsed(&seen)], &graph);

    assert!(!updates.has_changed().expect("feed still open"));
}

#[test]
fn feed_store_notifies_when_an_older_page_extends_the_feed() {
    let keys = Keys::generate();
    let graph = SocialGraph::new(keys.public_key());
    let mut store = FeedStore::new();
    let feed = store.open_feed(FeedSpec::MainFeed {
        viewer: Some(keys.public_key()),
    });
    store.ingest_first_page(feed, parsed_posts(&[video_note(&keys, "a", 50)]), &graph);
    let mut updates = store.subscribe(feed).expect("open feed subscribes");
    updates.borrow_and_update();

    store.begin_load_more(feed);
    store.ingest_older_page(feed, parsed_posts(&[video_note(&keys, "b", 40)]), &graph);

    assert!(updates.has_changed().expect("feed still open"));
}
