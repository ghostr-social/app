//! One canonical post per identity: duplicate event ids collapse, and
//! addressable revisions sharing `kind:pubkey:d` keep only the newest,
//! ties keeping the lexicographically smaller event id — mirrors
//! `_canonicalEvents` / `_isNewer` in
//! lib/features/video_catalog/data/ndk_video_remote_source.dart.

mod feed_support;

use feed_support::{addressable_video, parsed, parsed_posts, video_note};
use nostr_sdk::Keys;
use rust_lib_ghostr::discovery::feed_assembly::canonical_posts;

#[test]
fn feed_assembly_collapses_duplicate_event_ids() {
    let keys = Keys::generate();
    let event = video_note(&keys, "clip", 10);

    let posts = canonical_posts(vec![parsed(&event), parsed(&event)]);

    assert_eq!(posts.len(), 1);
}

#[test]
fn feed_assembly_keeps_only_the_newest_addressable_revision() {
    let keys = Keys::generate();
    let stale = addressable_video(&keys, "vid-1", "draft", 10);
    let fresh = addressable_video(&keys, "vid-1", "final", 20);

    let posts = canonical_posts(parsed_posts(&[fresh.clone(), stale]));

    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].event_id, fresh.id.to_hex());
}

#[test]
fn feed_assembly_breaks_addressable_ties_by_smaller_event_id() {
    let keys = Keys::generate();
    let left = addressable_video(&keys, "vid-1", "left", 10);
    let right = addressable_video(&keys, "vid-1", "right", 10);
    let winner = left.id.to_hex().min(right.id.to_hex());

    let posts = canonical_posts(parsed_posts(&[left, right]));

    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].event_id, winner);
}

#[test]
fn feed_assembly_keeps_addressable_posts_with_distinct_identifiers() {
    let keys = Keys::generate();
    let events = [
        addressable_video(&keys, "vid-1", "one", 10),
        addressable_video(&keys, "vid-2", "two", 10),
    ];

    let posts = canonical_posts(parsed_posts(&events));

    assert_eq!(posts.len(), 2);
}
