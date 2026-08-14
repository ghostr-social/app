use crate::demand_leases::DemandLeases;
use crate::tests::demand_lease_fixture::{binding, blocked, catalog, meta};
use ghostr_engine::{ByteRange, PostId};
use std::collections::{HashMap, HashSet};

#[test]
fn lease_is_removed_after_its_representation_changes() {
    let mut catalog = catalog(&["current"]);
    let stale = binding(&catalog, "current");
    let mut leases = DemandLeases::default();
    leases.apply(blocked(1, "current", stale, ByteRange::new(4, 8)));
    catalog.upsert(PostId::new("current"), meta("replacement"));
    let allowed = HashSet::from([PostId::new("current")]);

    let active = leases.reconcile(&allowed, &catalog, &HashMap::new());

    assert!(active.is_empty());
    assert_eq!(leases.len(), 0);
}
