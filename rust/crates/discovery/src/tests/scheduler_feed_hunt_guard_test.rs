use super::scheduler_support::{context, request};
use crate::scheduler_feeds::FeedBook;

#[test]
fn canonical_feed_never_enters_the_query_hunt() {
    let feed = context("main");
    let mut feeds = FeedBook::default();
    feeds.open(feed.clone(), request());

    assert!(feeds.hunt_action(&feed).is_none());
}
