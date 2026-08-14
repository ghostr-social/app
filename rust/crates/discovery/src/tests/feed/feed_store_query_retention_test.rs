//! Search and hashtag snapshots expose deeper history than canonical feeds,
//! while retaining an explicit finite session window.

use crate::content::social_graph::SocialGraph;
use crate::feed::spec::FeedSpec;
use crate::feed::store::{FeedStore, FEED_POST_RETENTION, QUERY_POST_RETENTION};
use crate::tests::feed_store_support::page;
use nostr_sdk::Keys;

const PAGE: u64 = 100;
const NEWEST: u64 = 10_000;

#[test]
fn query_feeds_keep_a_bounded_deep_history_window() {
    for spec in [
        FeedSpec::Search("ghost".to_owned()),
        FeedSpec::Hashtag("ghost".to_owned()),
    ] {
        let graph = SocialGraph::new(Keys::generate().public_key());
        let mut store = FeedStore::new();
        let feed = store.open_feed(spec);
        store.ingest_first_page(feed, query_page(NEWEST), &graph);

        for index in 1..=QUERY_POST_RETENTION as u64 / PAGE {
            store.begin_load_more_at(feed, None);
            store.ingest_older_page(feed, query_page(NEWEST - index * PAGE), &graph);
        }

        assert_eq!(store.posts(feed).len(), QUERY_POST_RETENTION);
        assert!(store.posts(feed).len() > FEED_POST_RETENTION);
    }
}

fn query_page(newest: u64) -> Vec<crate::content::parsing::ParsedVideoPost> {
    page(newest, PAGE)
        .into_iter()
        .map(|mut post| {
            post.hashtags.push("ghost".to_owned());
            post
        })
        .collect()
}
