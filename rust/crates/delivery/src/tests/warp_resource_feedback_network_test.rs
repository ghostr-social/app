use crate::tests::adaptive_plan_fixture::state;
use crate::tests::adaptive_plan_measurements::PlanMeasurements;
use crate::tests::adaptive_plan_runner::{run_with_measurements, PlanScenario};
use ghostr_engine::adaptive::StorageSnapshot;
use std::collections::HashMap;

#[test]
fn measured_network_load_drives_warp_feedback_instead_of_estimated_capacity() {
    let at_target = plan(500_000);
    let above_target = plan(1_000_000);

    assert_eq!(network_price(&at_target), 0);
    assert!(network_price(&above_target) > network_price(&at_target));
}

fn plan(measured_bytes_per_second: u64) -> crate::manager::plan::PlannedWork {
    run_with_measurements(
        PlanScenario {
            state: state(),
            buffer_ms: 20_000,
            bytes_per_second: 500_000,
            storage: StorageSnapshot::new(2_000_000_000, 0),
            present: HashMap::new(),
            packet_loss_bps: 0,
            in_flight: &[],
            connection_capacity: 4,
        },
        PlanMeasurements::new(measured_bytes_per_second, 0),
    )
}

fn network_price(work: &crate::manager::plan::PlannedWork) -> u64 {
    work.warp
        .as_ref()
        .expect("WARP decision")
        .prices
        .network_micros
}
