use crate::execution::relay_executor::profile_enrichment::{profile_plan, MAX_PROFILE_OCCURRENCES};
use nostr_sdk::{Event, EventBuilder, Keys, Kind, Timestamp};
use std::collections::BTreeSet;

#[test]
fn profile_enrichment_keeps_the_newest_visible_window_bounded() {
    let events: Vec<_> = (0..=MAX_PROFILE_OCCURRENCES)
        .map(|index| video(index as u64 + 1))
        .collect();
    let oldest_author = events[0].pubkey;
    let newest_author = events.last().expect("valid test fixture").pubkey;

    let plan = profile_plan(&events).expect("profile plan");
    let authors: BTreeSet<_> = plan
        .queries
        .iter()
        .flat_map(|query| query.filter.authors.iter().flatten().copied())
        .collect();

    assert_eq!(authors.len(), MAX_PROFILE_OCCURRENCES);
    assert!(plan.queries.iter().all(|query| {
        let count = query.filter.authors.as_ref().map_or(0, BTreeSet::len);
        query.filter.limit == Some(count)
    }));
    assert!(!authors.contains(&oldest_author));
    assert!(authors.contains(&newest_author));
}

fn video(index: u64) -> Event {
    let secret = format!("{index:064x}");
    let keys = Keys::parse(&secret).expect("deterministic key");
    EventBuilder::new(Kind::Custom(21), format!("https://cdn.example/{index}.mp4"))
        .custom_created_at(Timestamp::from(index))
        .sign_with_keys(&keys)
        .expect("signed video")
}
