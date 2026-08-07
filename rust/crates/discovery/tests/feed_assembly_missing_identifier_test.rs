//! Addressable posts without a published identifier fall back to event ID.

mod feed_support;

use feed_support::{addressable_video, parsed};
use nostr_sdk::Keys;
use ghostr_discovery::feed_assembly::post_coordinate;

#[test]
fn addressable_post_without_identifier_uses_event_identity() {
    let event = addressable_video(&Keys::generate(), "clip", "video", 10);
    let mut post = parsed(&event);
    post.published_identifier = None;

    let coordinate = post_coordinate(&post);

    assert_eq!(coordinate, post.event_id);
}
