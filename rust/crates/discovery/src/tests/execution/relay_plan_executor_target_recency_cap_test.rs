use crate::execution::relay_executor::target_enrichment::axiom_test_support::target_plan;
use crate::execution::relay_executor::target_enrichment::axiom_test_support::MAX_TARGET_LOOKUPS;

use nostr_sdk::{Event, EventBuilder, EventId, Keys, Kind, Tag, Timestamp};
use std::collections::BTreeSet;

#[test]
fn target_budget_keeps_the_newest_potential_feed_occurrences() {
    let creator = Keys::generate();
    let wrappers: Vec<_> = (0..=MAX_TARGET_LOOKUPS)
        .map(|index| wrapper(&creator, index as u64))
        .collect();
    let oldest_target = referenced_id(&wrappers[0]);

    let plan = target_plan(&wrappers).expect("target plan");
    let planned: BTreeSet<_> = plan
        .queries
        .iter()
        .flat_map(|query| query.filter.ids.iter().flatten().copied())
        .collect();

    assert_eq!(planned.len(), MAX_TARGET_LOOKUPS);
    assert!(!planned.contains(&oldest_target));
}

fn wrapper(creator: &Keys, created_at: u64) -> Event {
    let original = EventBuilder::new(
        Kind::Custom(21),
        format!("https://cdn.example/v{created_at}.mp4"),
    )
    .sign_with_keys(creator)
    .expect("original");
    EventBuilder::new(Kind::Custom(16), "")
        .tags([
            tag(&["e", &original.id.to_hex(), "wss://relay.example"]),
            tag(&["p", &creator.public_key().to_hex()]),
            tag(&["k", "21"]),
        ])
        .custom_created_at(Timestamp::from(created_at))
        .sign_with_keys(&Keys::generate())
        .expect("wrapper")
}

fn referenced_id(wrapper: &Event) -> EventId {
    let value = wrapper.tags.first().expect("e tag").as_slice();
    EventId::from_hex(&value[1]).expect("event id")
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
