use crate::adaptive::{
    AdaptivePlayabilityPolicy, ControlMode, FeedOffset, HlsBootstrapStage, HlsBootstrapState,
    HlsCandidateSnapshot, PlannerContext, ResourceFeedback, ResourceObservation, ViewProbability,
    WarpPlanner, WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;
use crate::PostId;

#[test]
fn unready_current_hls_enters_emergency_and_survives_high_shadow_prices() {
    let mut state = snapshot(0, 80_000_000, 0, 0);
    state.hls_candidates.push(current());
    let base = AdaptivePlayabilityPolicy.plan(&state);
    let context = PlannerContext::explicitly_unavailable(&state)
        .with_segmented_storage_available_bytes(u64::MAX)
        .with_feedback(ResourceFeedback {
            revision: 1,
            actual: ResourceObservation::new(u64::MAX, 0, 0, 0),
            target: ResourceObservation::new(1, 1, 1, 1),
            price_snapshot: None,
        });

    let decision = WarpPlanner::default().plan(WarpPlannerInput::new(
        &state,
        &base,
        &OriginModel::default(),
        &context,
    ));

    assert_eq!(base.mode, ControlMode::Emergency);
    assert!(decision.reserve.degraded);
    assert!(decision.selected.is_some());
}

fn current() -> HlsCandidateSnapshot {
    HlsCandidateSnapshot {
        post: PostId::new("p0"),
        feed_offset: FeedOffset::new(0),
        view_probability: ViewProbability::new(1.0).expect("valid test fixture"),
        startup_value_ms: 2_000,
        cursor: Default::default(),
        player_preparation: Default::default(),
        state: HlsBootstrapState::Pending {
            stage: HlsBootstrapStage::RootManifest,
            source: "https://hls.example/root.m3u8".into(),
        },
    }
}
