mod feed_support;

use feed_support::{addressable_video, specific_repost};
use ghostr_discovery::content::candidates::CandidateRegistry;
use nostr_sdk::Keys;

#[test]
fn direct_replay_does_not_replace_a_newer_specific_repost() {
    let creator = Keys::generate();
    let old = addressable_video(&creator, "clip", "old", 100);
    let current = addressable_video(&creator, "clip", "current", 200);
    let wrapper = specific_repost(&Keys::generate(), &old, 300);
    let mut registry = CandidateRegistry::new();

    registry.inspect_all(std::slice::from_ref(&current));
    registry.inspect_all(&[wrapper]);
    let replay = registry.inspect_all(&[current]);

    assert!(replay.admitted.is_empty());
}
