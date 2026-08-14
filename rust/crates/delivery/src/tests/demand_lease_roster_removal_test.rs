use crate::demand_leases::DemandLeases;
use crate::playback_demand::demand_channel;
use crate::tests::demand_lease_fixture::{binding, catalog};
use ghostr_engine::{ByteRange, PostId};
use std::collections::{HashMap, HashSet};

#[test]
fn live_lease_stays_dormant_while_its_post_leaves_the_roster() {
    let catalog = catalog(&["removed"]);
    let (sender, mut receiver) = demand_channel();
    let mut consumer = sender.consumer(PostId::new("removed"), Some(binding(&catalog, "removed")));
    let wanted = ByteRange::new(4, 8);
    consumer.demand(wanted);
    let mut leases = DemandLeases::default();
    leases.apply(receiver.try_recv().expect("initial blocked lease"));

    let active = leases.reconcile(&HashSet::new(), &catalog, &HashMap::new());
    assert!(active.is_empty());
    assert_eq!(leases.len(), 1, "open consumer remains dormant");
    assert!(receiver.try_recv().is_err(), "consumer does not re-emit");

    let restored = HashSet::from([PostId::new("removed")]);
    let active = leases.reconcile(&restored, &catalog, &HashMap::new());
    assert_eq!(active[&PostId::new("removed")], wanted);
}
