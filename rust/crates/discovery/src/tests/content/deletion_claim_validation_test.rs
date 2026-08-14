use crate::content::deletions::deletion_claims;
use nostr_sdk::{EventBuilder, Keys, Kind, Tag};

#[test]
fn only_author_valid_event_or_address_targets_become_claims() {
    let author = Keys::generate();
    let other = Keys::generate();
    let mut invalid = EventBuilder::new(Kind::EventDeletion, "delete")
        .sign_with_keys(&author)
        .expect("deletion");
    invalid.content.push_str(" tampered");
    let malformed = EventBuilder::new(Kind::EventDeletion, "delete")
        .tags([
            tag(&["p", &other.public_key().to_hex()]),
            tag(&["a", &format!("34235:{}:clip", other.public_key())]),
            tag(&["a", &format!("34235:{}:   ", author.public_key())]),
        ])
        .sign_with_keys(&author)
        .expect("deletion");

    assert!(deletion_claims(&[invalid, malformed]).is_empty());
}

fn tag(values: &[&str]) -> Tag {
    Tag::parse(
        values
            .iter()
            .map(|value| (*value).to_owned())
            .collect::<Vec<_>>(),
    )
    .expect("tag")
}
