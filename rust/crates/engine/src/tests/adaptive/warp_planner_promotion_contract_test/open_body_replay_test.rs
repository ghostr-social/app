use super::support::planned_with_model;
use crate::adaptive::{
    DecisionPrivacy, DecisionRecord, DecisionReplayStatus, ShadowPrices, WarpDecisionRecordInput,
};
use crate::origin_model::{
    MediaClass, OpenBodyObservation, OriginContext, OriginModel, OriginQuery, RequestMethod,
};

#[test]
fn private_planner_replay_round_trips_nonempty_open_body_evidence() {
    let mut origins = OriginModel::default();
    origins.observe_open_body(&OpenBodyObservation::success(query(), 1_000));
    let (snapshot, decision) = planned_with_model(Some(200_000), false, &origins);
    let record = DecisionRecord::capture_warp(WarpDecisionRecordInput {
        sequence: 92,
        snapshot: &snapshot,
        decision: &decision,
        legacy_shadow_prices: ShadowPrices::default(),
        models: &[],
        privacy: &DecisionPrivacy::from_key([31; 32]),
    });
    let encoded = serde_json::to_string(&record).expect("record serializes");

    assert!(encoded.contains("open_body_origins"));
    assert!(!encoded.contains("origin.example"));
    let restored: DecisionRecord = serde_json::from_str(&encoded).expect("record restores");
    assert_eq!(restored.integrity_status(), DecisionReplayStatus::Verified);
    assert_eq!(
        restored.search_integrity_status(),
        DecisionReplayStatus::Verified
    );
    assert_eq!(
        serde_json::to_string(&restored).expect("record serializes"),
        encoded
    );
}

fn query() -> OriginQuery {
    OriginQuery::new(
        "https://origin.example/media",
        OriginContext::new(RequestMethod::RangeGet, 200_000, MediaClass::ProgressiveMp4),
    )
}
