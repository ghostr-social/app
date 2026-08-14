use crate::demand_leases::DemandLeases;
use crate::tests::demand_lease_fixture::{binding, blocked, catalog};
use ghostr_engine::{ByteRange, PostId};
use std::collections::{HashMap, HashSet};

#[test]
fn prepared_next_lease_survives_promotion_to_current() {
    let catalog = catalog(&["old", "next", "after"]);
    let mut leases = DemandLeases::default();
    leases.apply(blocked(
        2,
        "next",
        binding(&catalog, "next"),
        ByteRange::new(12, 16),
    ));
    let promoted = HashSet::from([PostId::new("next"), PostId::new("after")]);

    let active = leases.reconcile(&promoted, &catalog, &HashMap::new());

    assert_eq!(active[&PostId::new("next")], ByteRange::new(12, 16));
    assert_eq!(leases.len(), 1);
}
