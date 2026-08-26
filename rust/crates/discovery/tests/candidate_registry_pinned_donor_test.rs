use crate::content::candidates::CandidateRegistry;
use crate::tests::feed_support::{addressable_video, specific_repost};
use nostr_sdk::Keys;

#[test]
fn newer_content_does_not_replace_a_more_recent_specific_pin() {
    let creator = Keys::generate();
    let old = addressable_video(&creator, "clip", "old", 100);
    let current = addressable_video(&creator, "clip", "current", 200);
    let newer = addressable_video(&creator, "clip", "newer", 250);
    let wrapper = specific_repost(&Keys::generate(), &old, 300);
    let mut registry = CandidateRegistry::new();

    registry.inspect_all(&[current]);
    registry.inspect_all(&[wrapper]);
    let donor = registry.inspect_all(&[newer]);

    assert!(donor.admitted.is_empty());
}
