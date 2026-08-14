use crate::execution::relay_executor::target_enrichment::target_plan;
use nostr_sdk::{Event, EventBuilder, EventId, Keys, Kind, Tag};
use std::collections::BTreeSet;

#[test]
fn every_distinct_creator_target_in_a_normal_page_is_planned() {
    let wrappers: Vec<_> = (0..17).map(|_| wrapper()).collect();
    let expected: BTreeSet<_> = wrappers
        .iter()
        .filter_map(|event| event.tags.iter().find_map(event_id))
        .collect();
    let plan = target_plan(&wrappers).expect("target plan");
    let planned: BTreeSet<_> = plan
        .queries
        .iter()
        .flat_map(|query| query.filter.ids.iter().flatten().copied())
        .collect();

    assert_eq!(planned, expected);
}

fn wrapper() -> Event {
    let creator = Keys::generate();
    let original = EventBuilder::new(Kind::TextNote, "https://cdn.example/v.mp4")
        .sign_with_keys(&creator)
        .expect("original");
    EventBuilder::new(Kind::Repost, "")
        .tags([
            tag(&["e", &original.id.to_hex(), "wss://relay.example"]),
            tag(&["p", &creator.public_key().to_hex()]),
        ])
        .sign_with_keys(&Keys::generate())
        .expect("wrapper")
}

fn event_id(tag: &Tag) -> Option<EventId> {
    let values = tag.as_slice();
    if values.first().map(String::as_str) != Some("e") {
        return None;
    }
    EventId::from_hex(values.get(1)?).ok()
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
