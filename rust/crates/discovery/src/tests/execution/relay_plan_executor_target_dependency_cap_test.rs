use crate::execution::relay_executor::target_enrichment::{
    target_plan_with_dependencies, MAX_TARGET_LOOKUPS,
};
use nostr_sdk::{Event, EventBuilder, EventId, Keys, Kind, Tag, Timestamp};

#[test]
fn target_dependencies_report_the_oldest_lookup_outside_the_admission_cap() {
    let creator = Keys::generate();
    let reposter = Keys::generate();
    let wrappers: Vec<_> = (0..=MAX_TARGET_LOOKUPS)
        .map(|index| wrapper(&creator, &reposter, index))
        .collect();
    let oldest = wrappers[0].id;

    let (_, _, unplanned) = target_plan_with_dependencies(&wrappers).expect("target plan");

    assert_eq!(unplanned.into_iter().collect::<Vec<_>>(), [oldest]);
}

fn wrapper(creator: &Keys, reposter: &Keys, index: usize) -> Event {
    let raw_target = format!("{index:064x}");
    let target = EventId::from_hex(&raw_target).expect("target id");
    EventBuilder::new(Kind::Custom(16), "")
        .tags([
            tag(&["e", &target.to_hex()]),
            tag(&["p", &creator.public_key().to_hex()]),
            tag(&["k", "21"]),
        ])
        .custom_created_at(Timestamp::from(index as u64))
        .sign_with_keys(reposter)
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
