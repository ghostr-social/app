use crate::content::candidates::CandidateRegistry;
use nostr_sdk::{EventBuilder, Keys, Timestamp};

#[test]
fn candidate_revision_index_never_exceeds_its_declared_bound() {
    let keys = Keys::generate();
    let mut registry = CandidateRegistry::with_retention(3);

    for created_at in 1..=5 {
        let event = EventBuilder::text_note(format!("https://cdn.example/{created_at}.mp4"))
            .custom_created_at(Timestamp::from(created_at))
            .sign_with_keys(&keys)
            .expect("video");
        registry.inspect(&event);
    }

    assert_eq!(registry.retained_coordinates(), 3);
}
