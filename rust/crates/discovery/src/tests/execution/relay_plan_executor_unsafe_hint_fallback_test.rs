use crate::execution::relay_executor::target_enrichment::axiom_test_support::target_plan;

use crate::query::search::RelayTarget;
use nostr_sdk::{EventBuilder, Keys, Kind, Tag};

#[test]
fn unsafe_but_syntactic_hint_falls_back_to_the_author_outbox() {
    let creator = Keys::generate();
    let original = EventBuilder::new(Kind::TextNote, "https://cdn.example/v.mp4")
        .sign_with_keys(&creator)
        .expect("original");
    let wrapper = EventBuilder::new(Kind::Repost, "")
        .tags([
            tag(&["e", &original.id.to_hex(), "wss://localhost./socket"]),
            tag(&["p", &creator.public_key().to_hex()]),
        ])
        .sign_with_keys(&Keys::generate())
        .expect("wrapper");

    let plan = target_plan(core::slice::from_ref(&wrapper)).expect("fallback plan");

    assert_eq!(plan.queries.len(), 1);
    assert_eq!(plan.queries[0].target, RelayTarget::OutboxRelays);
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
