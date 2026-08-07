//! Search and hashtag snapshots preserve every discovered row so native
//! pagination can expose history beyond the canonical-feed retention window.

use crate::feed_spec::FeedSpec;
use crate::feed_store::{FeedStore, FEED_POST_RETENTION};
use crate::social_graph::SocialGraph;
use crate::tests::feed_store_support::page;
use nostr_sdk::Keys;

const PAGE: u64 = 100;
const NEWEST: u64 = 10_000;

#[test]
fn query_feeds_keep_rows_beyond_the_canonical_retention_window() {
    for spec in [
        FeedSpec::Search("ghost".to_owned()),
        FeedSpec::Hashtag("ghost".to_owned()),
    ] {
        let graph = SocialGraph::new(Keys::generate().public_key());
        let mut store = FeedStore::new();
        let feed = store.open_feed(spec);
        store.ingest_first_page(feed, query_page(NEWEST), &graph);

        for index in 1..=FEED_POST_RETENTION as u64 / PAGE {
            store.begin_load_more(feed);
            store.ingest_older_page(feed, query_page(NEWEST - index * PAGE), &graph);
        }

        assert_eq!(store.posts(feed).len(), FEED_POST_RETENTION + PAGE as usize);
    }
}

fn query_page(newest: u64) -> Vec<crate::event_parsing::ParsedVideoPost> {
    page(newest, PAGE)
        .into_iter()
        .map(|mut post| {
            post.hashtags.push("ghost".to_owned());
            post
        })
        .collect()
}
