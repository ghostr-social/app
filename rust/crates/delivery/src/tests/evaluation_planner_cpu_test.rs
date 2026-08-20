use crate::tests::adaptive_plan_support::plan;
use ghostr_engine::adaptive::StorageSnapshot;

#[test]
fn each_real_planning_pass_carries_measured_cpu_time() {
    let work = plan(20_000, 4_000_000, StorageSnapshot::new(2_000_000_000, 0));

    assert!(work.planner_cpu_micros > 0);
}
