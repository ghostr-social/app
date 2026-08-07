//! A newer addressable revision deterministically replaces the stored row.

mod feed_support;

use feed_support::{addressable_video, parsed};
use nostr_sdk::Keys;
use ghostr_discovery::feed::assembly::canonical_posts;

#[test]
fn newer_revision_replaces_the_existing_coordinate() {
    let event = addressable_video(&Keys::generate(), "clip", "stale", 10);
    let mut stale = parsed(&event);
    stale.event_id = "f".repeat(64);
    let mut fresh = stale.clone();
    fresh.created_at = 20;
    fresh.event_id = "e".repeat(64);

    let posts = canonical_posts(vec![stale, fresh.clone()]);

    assert_eq!(posts, vec![fresh]);
}
