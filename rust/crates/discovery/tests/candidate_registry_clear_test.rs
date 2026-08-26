use crate::content::candidates::{CandidateAdmission, CandidateRegistry};
use crate::tests::feed_support::video_note;
use nostr_sdk::Keys;

#[test]
fn clearing_candidates_allows_the_same_coordinate_to_be_admitted_again() {
    let video = video_note(&Keys::generate(), "clip", 42);
    let mut registry = CandidateRegistry::new();
    assert!(matches!(
        registry.inspect(&video).admission,
        CandidateAdmission::Accepted(_)
    ));

    registry.clear();

    assert!(matches!(
        registry.inspect(&video).admission,
        CandidateAdmission::Accepted(_)
    ));
}
