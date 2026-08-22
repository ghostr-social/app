use crate::delivery_events::DeliveryNetworkStatus;
use crate::tests::adaptive_plan_fixture::state;
use crate::tests::adaptive_plan_runner::{run, PlanScenario};
use ghostr_engine::adaptive::StorageSnapshot;
use ghostr_engine::origin_model::NetworkClass;
use std::collections::HashMap;

#[test]
fn production_plan_maps_one_live_network_class_across_all_candidates() {
    let mut state = state();
    state.apply_network_status(DeliveryNetworkStatus::new(NetworkClass::Wifi, 1));
    let work = run(PlanScenario {
        state,
        buffer_ms: 20_000,
        bytes_per_second: 4_000_000,
        storage: StorageSnapshot::new(2_000_000_000, 0),
        present: HashMap::new(),
        packet_loss_bps: 0,
        in_flight: &[],
        connection_capacity: 4,
    });
    assert!(work.snapshot.expect("snapshot").candidates.len() > 1);
    assert_eq!(
        work.warp.expect("WARP decision").planner_network_class(),
        Some(NetworkClass::Wifi),
    );
}
