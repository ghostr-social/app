use crate::demand_leases::DemandLeases;
use crate::playback_demand::{demand_channel, DemandState};
use crate::tests::demand_lease_fixture::{binding, catalog};
use ghostr_engine::{ByteRange, PostId};
use std::collections::{HashMap, HashSet};

#[test]
fn prepared_lease_reactivates_without_consumer_reemission() {
    let catalog = catalog(&["current", "ahead"]);
    let (sender, mut receiver) = demand_channel();
    let mut consumer = sender.consumer(PostId::new("ahead"), Some(binding(&catalog, "ahead")));
    let wanted = ByteRange::new(12, 16);
    consumer.demand(wanted);
    let mut leases = DemandLeases::default();
    let emitted = receiver.try_recv().expect("initial blocked lease");
    assert!(matches!(emitted, DemandState::Blocked(_)));
    leases.apply(emitted);

    let foreground = HashSet::from([PostId::new("current")]);
    assert!(leases
        .reconcile(&foreground, &catalog, &HashMap::new())
        .is_empty());
    assert_eq!(leases.len(), 1, "roster lease remains dormant");
    assert!(receiver.try_recv().is_err(), "consumer does not re-emit");

    let promoted = HashSet::from([PostId::new("ahead")]);
    let active = leases.reconcile(&promoted, &catalog, &HashMap::new());
    assert_eq!(active[&PostId::new("ahead")], wanted);
}
