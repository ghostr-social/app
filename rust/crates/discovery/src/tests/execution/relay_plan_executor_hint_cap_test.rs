use crate::execution::relay_executor::target_enrichment::axiom_test_support::target_plan;

use crate::query::search::RelayTarget;
use nostr_sdk::{Event, EventBuilder, Keys, Kind, Tag};
use std::collections::BTreeSet;

#[test]
fn repost_target_plan_caps_distinct_untrusted_relay_hints() {
    let wrappers: Vec<_> = (0..401).map(wrapper_with_unique_hint).collect();

    let plan = target_plan(&wrappers).expect("target lookup");
    let hints: BTreeSet<_> = plan
        .queries
        .iter()
        .filter_map(|query| match &query.target {
            RelayTarget::HintedRelays(hints) => Some(hints.iter()),
            _ => None,
        })
        .flatten()
        .collect();

    assert_eq!(hints.len(), 400);
}

fn wrapper_with_unique_hint(index: usize) -> Event {
    let creator = Keys::generate();
    let original = EventBuilder::new(Kind::Custom(21), "https://cdn.example/v.mp4")
        .sign_with_keys(&creator)
        .expect("original");
    EventBuilder::new(Kind::Custom(16), "")
        .tags([
            tag(&[
                "e",
                &original.id.to_hex(),
                &format!("wss://relay{index}.example"),
            ]),
            tag(&["p", &creator.public_key().to_hex()]),
            tag(&["k", "21"]),
        ])
        .sign_with_keys(&Keys::generate())
        .expect("wrapper")
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
