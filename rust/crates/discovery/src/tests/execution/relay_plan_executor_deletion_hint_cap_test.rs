use crate::execution::relay_executor::deletion_enrichment::axiom_test_support::deletion_plan;

use crate::query::search::RelayTarget;
use nostr_sdk::{EventBuilder, JsonUtil as _, Keys, Kind, Tag};

#[test]
fn repeated_reposts_cannot_expand_one_deletion_target_past_four_hints() {
    let creator = Keys::generate();
    let original = EventBuilder::new(Kind::TextNote, "https://cdn.example/v.mp4")
        .sign_with_keys(&creator)
        .expect("original");
    let wrappers = (0..5)
        .map(|index| {
            EventBuilder::new(Kind::Repost, original.as_json())
                .tags([
                    tag(&[
                        "e",
                        &original.id.to_hex(),
                        &format!("wss://relay-{index}.example"),
                    ]),
                    tag(&["p", &creator.public_key().to_hex()]),
                ])
                .sign_with_keys(&Keys::generate())
                .expect("wrapper")
        })
        .collect::<Vec<_>>();

    let plan = deletion_plan(&wrappers).expect("deletion plan");
    let largest = plan
        .queries
        .iter()
        .filter_map(|query| match &query.target {
            RelayTarget::HintedRelays(hints) => Some(hints.len()),
            _ => None,
        })
        .max();

    assert_eq!(largest, Some(4));
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
