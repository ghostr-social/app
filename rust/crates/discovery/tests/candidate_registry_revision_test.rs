mod feed_support;

use feed_support::addressable_video;
use nostr_sdk::Keys;
use ghostr_discovery::content::candidates::{CandidateAdmission, CandidateRegistry};

#[test]
fn a_new_addressable_revision_replaces_the_same_candidate() {
    let keys = Keys::generate();
    let first = addressable_video(&keys, "dance", "first", 20);
    let stale = addressable_video(&keys, "dance", "stale", 10);
    let latest = addressable_video(&keys, "dance", "latest", 30);
    let mut registry = CandidateRegistry::new();

    let CandidateAdmission::Accepted(first) = registry.admit(&first) else {
        panic!("first revision should be admitted");
    };
    assert_eq!(registry.admit(&stale), CandidateAdmission::Duplicate);
    let CandidateAdmission::Replaced(latest) = registry.admit(&latest) else {
        panic!("newest revision should replace the candidate");
    };

    assert_eq!(latest.id, first.id);
    assert_eq!(latest.post.meta.urls, ["https://cdn.example/latest.mp4"]);
    assert_eq!(registry.len(), 1);
}
