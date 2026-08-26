use crate::content::candidates::CandidateRegistry;
use crate::tests::feed_support::{addressable_video, specific_repost};
use nostr_sdk::Keys;

#[test]
fn specific_repost_readmits_its_visible_older_revision() {
    let creator = Keys::generate();
    let reposter = Keys::generate();
    let old = addressable_video(&creator, "clip", "old", 100);
    let current = addressable_video(&creator, "clip", "current", 200);
    let wrapper = specific_repost(&reposter, &old, 300);
    let mut registry = CandidateRegistry::new();

    registry.inspect_all(&[current]);
    let admitted = registry.inspect_all(&[wrapper]).admitted;

    assert_eq!(admitted.len(), 1);
    assert_eq!(admitted[0].post.event_id, old.id.to_hex());
}
