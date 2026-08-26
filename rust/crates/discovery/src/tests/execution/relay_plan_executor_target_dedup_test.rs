use crate::execution::relay_executor::target_enrichment::axiom_test_support::target_plan;

use nostr_sdk::{Event, EventBuilder, Keys, Kind, Tag};
use std::collections::BTreeSet;

#[test]
fn duplicate_target_references_do_not_starve_a_later_distinct_target() {
    let first = original();
    let second = original();
    let mut wrappers: Vec<_> = (0..400).map(|_| wrapper(&first)).collect();
    wrappers.push(wrapper(&second));

    let plan = target_plan(&wrappers).expect("target plan");
    let planned: BTreeSet<_> = plan
        .queries
        .iter()
        .flat_map(|query| query.filter.ids.iter().flatten().copied())
        .collect();

    assert_eq!(planned, BTreeSet::from([first.id, second.id]));
}

fn original() -> Event {
    EventBuilder::new(Kind::TextNote, "https://cdn.example/v.mp4")
        .sign_with_keys(&Keys::generate())
        .expect("original")
}

fn wrapper(original: &Event) -> Event {
    EventBuilder::new(Kind::Repost, "")
        .tags([
            tag(&["e", &original.id.to_hex(), "wss://relay.example"]),
            tag(&["p", &original.pubkey.to_hex()]),
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
    .expect("tag")
}
