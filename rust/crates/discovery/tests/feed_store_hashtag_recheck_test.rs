//! Hashtag feeds recheck the tag on every fetched post — relay tag
//! matching is not trusted the way NIP-50 text judgement is — mirrors
//! `_selectPosts` in
//! lib/features/video_catalog/domain/discovery_video_search_repository.dart.

mod feed_support;

use feed_support::{hashtag_video_note, parsed_posts, video_note};
use ghostr_discovery::content::social_graph::SocialGraph;
use ghostr_discovery::feed::spec::FeedSpec;
use ghostr_discovery::feed::store::FeedStore;
use nostr_sdk::Keys;

#[test]
fn feed_store_hashtag_feed_keeps_only_posts_carrying_the_tag() {
    let creator = Keys::generate();
    let graph = SocialGraph::new(Keys::generate().public_key());
    let mut store = FeedStore::new();
    let feed = store.open_feed(FeedSpec::Hashtag("#cats".to_owned()));

    let fetched = parsed_posts(&[
        hashtag_video_note(&creator, "tagged", "cats", 30),
        video_note(&creator, "untagged", 20),
        hashtag_video_note(&creator, "other-tag", "dogs", 10),
    ]);
    store.ingest_first_page(feed, fetched, &graph);

    let slugs: Vec<&str> = store
        .posts(feed)
        .iter()
        .map(|post| post.meta.urls[0].as_str())
        .collect();
    assert_eq!(slugs, ["https://cdn.example/tagged.mp4"]);
}

#[test]
fn feed_store_search_for_a_hashtag_rechecks_the_tag_too() {
    // A "#tag" search runs the hashtag branch of `searchVideos`, so the
    // same recheck applies to it.
    let creator = Keys::generate();
    let graph = SocialGraph::new(Keys::generate().public_key());
    let mut store = FeedStore::new();
    let feed = store.open_feed(FeedSpec::Search("#Cats".to_owned()));

    let fetched = parsed_posts(&[
        hashtag_video_note(&creator, "tagged", "CATS", 30),
        video_note(&creator, "untagged", 20),
    ]);
    store.ingest_first_page(feed, fetched, &graph);

    assert_eq!(store.posts(feed).len(), 1);
    assert_eq!(store.posts(feed)[0].hashtags, ["cats".to_owned()]);
}
