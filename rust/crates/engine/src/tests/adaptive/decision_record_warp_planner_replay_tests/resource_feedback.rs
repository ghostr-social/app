use super::support::record;
use crate::adaptive::{
    AdaptivePlayabilityPolicy, DecisionRecord, DecisionReplayStatus, PlannerContext,
    ResourceFeedback, ResourceFeedbackCursor, ResourceObservation, ResourcePriceSnapshot,
    ResourcePrices, WarpPlanner, WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;

#[test]
fn authoritative_resource_snapshot_survives_json_and_exact_replay() {
    let state = snapshot(2, 20_000_000, 8_000, 18);
    let base = AdaptivePlayabilityPolicy.plan(&state);
    let feedback = ResourceFeedback::authoritative(
        ResourcePriceSnapshot::new(ResourceFeedbackCursor::new(3, 7), prices()),
        ResourceObservation::new(11, 22, 33, 44),
        ResourceObservation::new(55, 66, 77, 88),
    );
    let context = PlannerContext::explicitly_unavailable(&state).with_feedback(feedback);
    let decision = WarpPlanner::default().plan(WarpPlannerInput::new(
        &state,
        &base,
        &OriginModel::default(),
        &context,
    ));
    let json = serde_json::to_string(&record(&state, &decision)).unwrap();
    let restored: DecisionRecord = serde_json::from_str(&json).unwrap();

    assert!(json.contains("\"cursor\":{\"epoch\":3,\"revision\":7}"));
    assert!(json.contains("\"network_micros\":101"));
    assert_eq!(restored.replay(), DecisionReplayStatus::Verified);
    assert!(restored.replay_warp_search().is_ok());
}

fn prices() -> ResourcePrices {
    ResourcePrices {
        network_micros: 101,
        storage_micros: 202,
        cpu_micros: 303,
        request_micros: 404,
    }
}
