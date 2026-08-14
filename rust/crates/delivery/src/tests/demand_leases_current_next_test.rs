use crate::demand_leases::DemandLeases;
use crate::tests::demand_lease_fixture::{binding, blocked, catalog};
use ghostr_engine::{ByteRange, PostId};
use std::collections::{HashMap, HashSet};

#[test]
fn simultaneous_current_and_next_demand_remain_active() {
    let catalog = catalog(&["current", "next"]);
    let mut leases = DemandLeases::default();
    leases.apply(blocked(
        1,
        "current",
        binding(&catalog, "current"),
        ByteRange::new(4, 8),
    ));
    leases.apply(blocked(
        2,
        "next",
        binding(&catalog, "next"),
        ByteRange::new(12, 16),
    ));
    let allowed = HashSet::from([PostId::new("current"), PostId::new("next")]);

    let active = leases.reconcile(&allowed, &catalog, &HashMap::new());

    assert_eq!(active[&PostId::new("current")], ByteRange::new(4, 8));
    assert_eq!(active[&PostId::new("next")], ByteRange::new(12, 16));
}
