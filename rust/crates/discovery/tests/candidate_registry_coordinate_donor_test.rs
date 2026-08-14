mod feed_support;

use feed_support::{addressable_video, repost};
use ghostr_discovery::content::candidates::CandidateRegistry;
use nostr_sdk::Keys;

#[test]
fn coordinate_repost_reprioritizes_the_latest_revision() {
    let creator = Keys::generate();
    let stale = addressable_video(&creator, "clip", "stale", 100);
    let current = addressable_video(&creator, "clip", "current", 200);
    let wrapper = repost(&Keys::generate(), &stale, 400);
    let mut registry = CandidateRegistry::new();

    registry.inspect_all(std::slice::from_ref(&current));
    let admitted = registry.inspect_all(&[wrapper]).admitted;

    assert_eq!(admitted.len(), 1);
    assert_eq!(admitted[0].post.event_id, current.id.to_hex());
    assert_eq!(admitted[0].post.feed_sort_at, 400);
}
