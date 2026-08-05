use crate::discovery::event_parsing::ParsedVideoPost;
use crate::discovery::feed_spec::FeedSpec;
use crate::discovery::pagination::next_page_cursor;
use nostr_sdk::Timestamp;

pub(crate) fn post_cursor(posts: &[ParsedVideoPost]) -> Option<Timestamp> {
    next_page_cursor(posts.iter().map(|post| Timestamp::from(post.created_at)))
}

pub(crate) fn older_cursor(
    spec: &FeedSpec,
    current: Option<Timestamp>,
    fetched: &[ParsedVideoPost],
) -> Option<Timestamp> {
    match post_cursor(fetched) {
        Some(next) => Some(next),
        None if spec.exhausts_on_empty_page() => None,
        None => current,
    }
}
