use crate::content::deletions::deletion_claims;
use nostr_sdk::{EventBuilder, Keys, Kind, Tag};

#[test]
fn one_deletion_event_cannot_expand_beyond_the_claim_bound() {
    let tags = (0..501)
        .map(|index| Tag::parse(vec!["e".to_owned(), format!("target-{index}")]).expect("tag"));
    let event = EventBuilder::new(Kind::EventDeletion, "delete")
        .tags(tags)
        .sign_with_keys(&Keys::generate())
        .expect("deletion");

    assert_eq!(deletion_claims(&[event]).len(), 500);
}
