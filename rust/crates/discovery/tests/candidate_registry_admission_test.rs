mod feed_support;

use feed_support::video_note;
use nostr_sdk::{EventBuilder, Keys, Kind};
use ghostr_discovery::content::candidates::{CandidateAdmission, CandidateRegistry};

#[test]
fn raw_events_are_parsed_validated_and_deduplicated_once() {
    let keys = Keys::generate();
    let video = video_note(&keys, "clip", 42);
    let invalid = EventBuilder::new(Kind::Metadata, "{}")
        .sign_with_keys(&keys)
        .expect("signed metadata");
    let mut registry = CandidateRegistry::new();
    assert!(registry.is_empty());

    let CandidateAdmission::Accepted(candidate) = registry.admit(&video) else {
        panic!("video should be admitted");
    };

    assert_eq!(candidate.post.meta.urls, ["https://cdn.example/clip.mp4"]);
    assert!(candidate
        .id
        .as_str()
        .chars()
        .all(|char| char.is_ascii_hexdigit()));
    assert_eq!(registry.admit(&video), CandidateAdmission::Duplicate);
    assert_eq!(registry.admit(&invalid), CandidateAdmission::Rejected);
    assert_eq!(registry.admit(&invalid), CandidateAdmission::Rejected);
    assert_eq!(registry.len(), 1);
    assert!(!registry.is_empty());
}
