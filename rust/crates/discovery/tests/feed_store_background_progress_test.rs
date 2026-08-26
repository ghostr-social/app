use crate::content::social_graph::SocialGraph;
use crate::feed::spec::FeedSpec;
use crate::feed::store::FeedStore;
use crate::tests::feed_support::{parsed, video_note};
use nostr_sdk::{Keys, Timestamp};

#[test]
fn background_progress_settles_without_moving_the_historical_cursor() {
    let keys = Keys::generate();
    let graph = SocialGraph::new(keys.public_key());
    let mut store = FeedStore::new();
    let feed = store.open_feed(FeedSpec::MainFeed {
        viewer: Some(keys.public_key()),
    });
    store.set_retrieval_cursor(feed, Some(Timestamp::from(50)));
    store.begin_background_load(feed);
    assert!(store.begin_load_more_at(feed, None).is_none());

    let progress = parsed(&video_note(&keys, "progress", 90));
    assert!(store.ingest_progress(feed, progress, &graph));
    let head = parsed(&video_note(&keys, "head", 100));
    assert!(store.ingest_head_page(feed, vec![head], &graph));

    assert_eq!(
        store.begin_load_more_at(feed, None),
        Some(Timestamp::from(50))
    );
}
