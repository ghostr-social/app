use crate::demand_leases::DemandLeases;
use crate::tests::demand_lease_fixture::{binding, blocked, catalog};
use ghostr_engine::{ByteRange, PostId};
use std::collections::{HashMap, HashSet};

#[test]
fn fulfilled_lease_reactivates_if_its_covered_bytes_disappear() {
    let catalog = catalog(&["current"]);
    let mut leases = DemandLeases::default();
    leases.apply(blocked(
        1,
        "current",
        binding(&catalog, "current"),
        ByteRange::new(4, 8),
    ));
    let allowed = HashSet::from([PostId::new("current")]);
    let present = HashMap::from([(PostId::new("current"), vec![ByteRange::new(0, 8)])]);

    let active = leases.reconcile(&allowed, &catalog, &present);

    assert!(active.is_empty());
    assert_eq!(leases.len(), 1, "covered lease remains dormant");

    let evicted = HashMap::from([(PostId::new("current"), vec![ByteRange::new(0, 4)])]);
    let active = leases.reconcile(&allowed, &catalog, &evicted);

    assert_eq!(active[&PostId::new("current")], ByteRange::new(4, 8));
}
