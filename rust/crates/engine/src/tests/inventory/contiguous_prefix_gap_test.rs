use crate::inventory_controller::Mode;
use crate::tests::critical_prefix_support::{bench, mark_startable};
use crate::tiers::Tier;
use std::collections::BTreeSet;

#[test]
fn gaps_keep_the_critical_prefix_hungry_and_exclusive() {
    let mut bench = bench();
    mark_startable(&mut bench, &["current", "next3", "next4", "next5"]);

    let inventory = bench.observe();
    assert_eq!(inventory.mode, Mode::Hunger);
    assert_eq!(inventory.counts.startable, 1);
    assert!(inventory.counts.startable < inventory.counts.target);

    let requests = bench.run();
    let requested: BTreeSet<_> = requests
        .iter()
        .map(|request| request.chunk.post.as_str())
        .collect();
    assert_eq!(requested, BTreeSet::from(["next1", "next2"]));
    assert!(requests
        .iter()
        .all(|request| request.tier == Tier::T2Startability));
}
