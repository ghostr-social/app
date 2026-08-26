use crate::execution::relay_executor::profile_enrichment::{
    profile_plan, MAX_PROFILE_AUTHORS, MAX_PROFILE_AUTHORS_PER_QUERY, MAX_PROFILE_OCCURRENCES,
};
use nostr_sdk::{Event, EventBuilder, JsonUtil as _, Keys, Kind, Timestamp};
use std::collections::BTreeSet;

#[test]
fn retained_reposts_bound_creator_and_reposter_profile_queries() {
    let wrappers: Vec<_> = (0..MAX_PROFILE_OCCURRENCES)
        .map(|index| wrapper(index as u64 + 1))
        .collect();

    let plan = profile_plan(&wrappers).expect("profile plan");
    let authors: BTreeSet<_> = plan
        .queries
        .iter()
        .flat_map(|query| query.filter.authors.iter().flatten().copied())
        .collect();

    assert_eq!(authors.len(), MAX_PROFILE_AUTHORS);
    assert!(plan.queries.len() <= 8);
    assert!(plan.queries.iter().all(|query| {
        query.filter.authors.as_ref().is_some_and(|values| {
            values.len() <= MAX_PROFILE_AUTHORS_PER_QUERY
                && query.filter.limit == Some(values.len())
        })
    }));
}

fn wrapper(index: u64) -> Event {
    let creator = keys(index);
    let reposter = keys(index + MAX_PROFILE_OCCURRENCES as u64);
    let original = EventBuilder::new(Kind::Custom(21), format!("https://cdn.example/{index}.mp4"))
        .sign_with_keys(&creator)
        .expect("signed video");
    EventBuilder::new(Kind::Custom(16), original.as_json())
        .custom_created_at(Timestamp::from(index))
        .sign_with_keys(&reposter)
        .expect("signed repost")
}

fn keys(index: u64) -> Keys {
    let secret = format!("{index:064x}");
    Keys::parse(&secret).expect("deterministic key")
}
