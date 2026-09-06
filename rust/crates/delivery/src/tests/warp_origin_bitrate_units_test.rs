use crate::delivery_events::DeliveryNetworkStatus;
use crate::tests::adaptive_plan_fixture::{source, state};
use crate::tests::adaptive_plan_runner::{run, PlanScenario};
use ghostr_engine::adaptive::StorageSnapshot;
use ghostr_engine::origin_model::{
    DecisionMode, MediaClass, NetworkClass, OriginContext, OriginModel, OriginQuery, RequestMethod,
};
use std::collections::HashMap;

#[test]
fn production_planner_preserves_origin_bits_per_second_without_eightfold_inflation() {
    let mut state = state();
    state.apply_network_status(DeliveryNetworkStatus::new(NetworkClass::Wifi, 1));
    let work = run(PlanScenario {
        state,
        buffer_ms: 1_000,
        bytes_per_second: 4_000_000,
        storage: StorageSnapshot::new(2_000_000_000, 0),
        present: HashMap::new(),
        packet_loss_bps: 0,
        in_flight: &[],
        connection_capacity: 2,
    });
    let query = OriginQuery::new(
        source(0),
        OriginContext::new(
            RequestMethod::RangeGet,
            256 * 1024,
            MediaClass::ProgressiveMp4,
        )
        .with_network(NetworkClass::Wifi)
        .with_observed_at_ms(1_000),
    );
    let estimate = OriginModel::default().estimate(&query, 1_000, DecisionMode::Safety);
    let snapshot = work.snapshot.expect("production planning snapshot");
    assert!(!snapshot.candidates.is_empty());
    for candidate in snapshot.candidates {
        assert_eq!(
            candidate.origins[0].throughput_bps, estimate.throughput_bps.selected,
            "origin model already reports bits/s; inflating it understates continuation cost"
        );
    }
    assert_eq!(snapshot.network.throughput_bps, 32_000_000);
}
