use crate::tests::adaptive_plan_fixture::state;
use crate::tests::adaptive_plan_measurements::PlanMeasurements;
use crate::tests::adaptive_plan_runner::{run_with_measurements, PlanScenario};
use ghostr_engine::adaptive::StorageSnapshot;
use std::collections::HashMap;

#[test]
fn capacity_revision_changes_the_warp_seed_when_storage_totals_are_equal() {
    let first = plan(7);
    let revised = plan(8);

    assert_ne!(seed(&first), seed(&revised));
}

fn plan(capacity_revision: u64) -> crate::manager::plan::PlannedWork {
    run_with_measurements(
        PlanScenario {
            state: state(),
            buffer_ms: 20_000,
            bytes_per_second: 500_000,
            storage: StorageSnapshot::new(800, 200),
            present: HashMap::new(),
            packet_loss_bps: 0,
            in_flight: &[],
            connection_capacity: 4,
        },
        PlanMeasurements::new(0, capacity_revision),
    )
}

fn seed(work: &crate::manager::plan::PlannedWork) -> u64 {
    work.warp
        .as_ref()
        .expect("WARP decision")
        .common_random_seed
}
