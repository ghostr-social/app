use crate::api::delivery::candidates::delivery_candidate;
use crate::discovery::content::candidates::CandidateRegistry;
use nostr_sdk::{EventBuilder, JsonUtil as _, Keys, Kind, Tag, Timestamp};

#[test]
fn repost_candidate_priority_uses_the_occurrence_time() {
    let creator = Keys::generate();
    let reposter = Keys::generate();
    let original = EventBuilder::new(Kind::TextNote, "https://cdn.example/v.mp4")
        .custom_created_at(Timestamp::from(10))
        .sign_with_keys(&creator)
        .expect("original");
    let wrapper = EventBuilder::new(Kind::Repost, original.as_json())
        .tags([
            tag(&["e", &original.id.to_hex(), "wss://relay.example"]),
            tag(&["p", &creator.public_key().to_hex()]),
        ])
        .custom_created_at(Timestamp::from(100))
        .sign_with_keys(&reposter)
        .expect("wrapper");
    let candidate = CandidateRegistry::new()
        .inspect_all(&[wrapper])
        .admitted
        .pop()
        .expect("candidate");

    assert_eq!(delivery_candidate(candidate).discovered_at, 100);
}

fn tag(values: &[&str]) -> Tag {
    Tag::parse(
        values
            .iter()
            .map(|value| (*value).to_owned())
            .collect::<Vec<_>>(),
    )
    .expect("valid tag")
}
