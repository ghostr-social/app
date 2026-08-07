//! Assembled feed pages order newest-first by created_at with ascending
//! event-ID tiebreak.

mod feed_support;

use feed_support::{parsed_posts, video_note};
use nostr_sdk::Keys;
use ghostr_discovery::feed::assembly::canonical_posts;

#[test]
fn feed_assembly_orders_posts_newest_first() {
    let keys = Keys::generate();
    let events = [
        video_note(&keys, "middle", 20),
        video_note(&keys, "oldest", 10),
        video_note(&keys, "newest", 30),
    ];

    let posts = canonical_posts(parsed_posts(&events));

    let created: Vec<u64> = posts.iter().map(|post| post.created_at).collect();
    assert_eq!(created, [30, 20, 10]);
}

#[test]
fn feed_assembly_breaks_created_at_ties_by_ascending_event_id() {
    let keys = Keys::generate();
    let events = [
        video_note(&keys, "one", 25),
        video_note(&keys, "two", 25),
        video_note(&keys, "three", 25),
    ];

    let posts = canonical_posts(parsed_posts(&events));

    let ids: Vec<&str> = posts.iter().map(|post| post.event_id.as_str()).collect();
    let mut expected: Vec<String> = events.iter().map(|event| event.id.to_hex()).collect();
    expected.sort();
    assert_eq!(ids, expected);
}
