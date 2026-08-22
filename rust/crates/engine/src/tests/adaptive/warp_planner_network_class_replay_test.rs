use super::network_class_support::fixture;
use crate::adaptive::{
    DecisionPrivacy, DecisionRecord, DecisionReplayStatus, ShadowPrices, WarpDecisionRecordInput,
    WarpPlanner, WarpPlannerInput,
};
use crate::origin_model::NetworkClass;

#[test]
fn progressive_network_class_is_replayable_and_mutation_evident() {
    let fixture = fixture(NetworkClass::Wifi);
    let decision = WarpPlanner::default().plan(WarpPlannerInput::new(
        &fixture.snapshot,
        &fixture.base,
        &fixture.origins,
        &fixture.context,
    ));
    let record = DecisionRecord::capture_warp(WarpDecisionRecordInput {
        sequence: 81,
        snapshot: &fixture.snapshot,
        decision: &decision,
        legacy_shadow_prices: ShadowPrices::default(),
        models: &[],
        privacy: &DecisionPrivacy::from_key([23; 32]),
    });
    assert!(record.replay_warp_search().is_ok());
    let mut json = serde_json::to_value(record).unwrap();
    let context = &mut json["warp_decision"]["planner_replay_capsule"]["context"];
    assert_eq!(context["network_class"], "Wifi");
    context["network_class"] = serde_json::json!("Cellular");
    let tampered: DecisionRecord = serde_json::from_value(json).unwrap();
    assert_eq!(tampered.replay(), DecisionReplayStatus::PlanMismatch);
}
