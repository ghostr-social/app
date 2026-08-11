use crate::tests::critical_prefix_support::bench;
use crate::tiers::Tier;
use std::collections::BTreeSet;

#[test]
fn an_empty_feed_only_schedules_the_target_sized_prefix() {
    let requests = bench().run();
    let requested: BTreeSet<_> = requests
        .iter()
        .map(|request| request.chunk.post.as_str())
        .collect();

    assert_eq!(
        requested,
        BTreeSet::from(["current", "next1", "next2", "next3"])
    );
    assert!(requests
        .iter()
        .all(|request| request.tier == Tier::T2Startability));
}
