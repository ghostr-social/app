use crate::content::candidates::{CandidateAdmission, CandidateRegistry};
use crate::tests::feed_support::video_note;
use nostr_sdk::{EventBuilder, Keys, Kind};

#[test]
fn raw_events_are_validated_and_deduplicated_at_admission() {
    let keys = Keys::generate();
    let video = video_note(&keys, "clip", 42);
    let invalid = EventBuilder::new(Kind::Metadata, "{}")
        .sign_with_keys(&keys)
        .expect("signed metadata");
    let mut registry = CandidateRegistry::new();
    let CandidateAdmission::Accepted(candidate) = registry.inspect(&video).admission else {
        panic!("video should be admitted");
    };

    assert_eq!(candidate.post.meta.urls, ["https://cdn.example/clip.mp4"]);
    assert!(candidate
        .id
        .as_str()
        .chars()
        .all(|char| char.is_ascii_hexdigit()));
    assert_eq!(
        registry.inspect(&video).admission,
        CandidateAdmission::Duplicate
    );
    assert_eq!(
        registry.inspect(&invalid).admission,
        CandidateAdmission::Rejected
    );
    assert_eq!(
        registry.inspect(&invalid).admission,
        CandidateAdmission::Rejected
    );
}
