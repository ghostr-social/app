use crate::adaptive::{
    ActionKind, AdaptivePlayabilityPolicy, FeedOffset, HlsBootstrapStage, HlsBootstrapState,
    HlsCandidateSnapshot, PlannerCommand, PlannerContext, ResourceCost, ViewProbability,
    WarpPlanner, WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;
use crate::PostId;

const SOURCE: &str = "https://hls.example/root.m3u8";

#[test]
fn current_hls_bootstrap_is_one_budgeted_receding_horizon_commitment() {
    let mut state = snapshot(0, 80_000_000, 0, 0);
    state
        .hls_candidates
        .push(candidate(HlsBootstrapStage::RootManifest));
    let base = AdaptivePlayabilityPolicy.plan(&state);
    let context = PlannerContext::explicitly_unavailable(&state)
        .with_segmented_storage_available_bytes(u64::MAX);
    let mut planner = WarpPlanner::default();
    let decision = planner.plan(WarpPlannerInput::new(
        &state,
        &base,
        &OriginModel::default(),
        &context,
    ));
    let selected = decision.selected.expect("current HLS manifest selected");
    assert_eq!(
        selected.node.kind,
        ActionKind::HlsBootstrap {
            stage: HlsBootstrapStage::RootManifest,
            cursor: Default::default(),
            maximum_bytes: crate::adaptive::REQUEST_SLICE_BYTES,
        }
    );
    assert_eq!(
        selected.node.resources,
        ResourceCost::new(
            crate::adaptive::REQUEST_SLICE_BYTES,
            crate::adaptive::REQUEST_SLICE_BYTES,
            0,
            1,
        )
    );
    assert!(matches!(
        selected.command,
        PlannerCommand::FetchHlsBootstrap {
            stage: HlsBootstrapStage::RootManifest,
            maximum_bytes: crate::adaptive::REQUEST_SLICE_BYTES,
            committed_until_ms: 13_000,
            ..
        }
    ));
    assert_eq!(decision.generated.actions.len(), 1);
}

#[test]
fn every_hls_stage_commits_one_bounded_network_and_storage_block() {
    let cases = [
        HlsBootstrapStage::RootManifest,
        HlsBootstrapStage::ChildPlaylist,
        HlsBootstrapStage::Initialization,
        HlsBootstrapStage::FirstSegment,
    ];
    for stage in cases {
        let mut state = snapshot(0, 80_000_000, 0, 0);
        state.hls_candidates.push(candidate(stage));
        let generated = crate::adaptive::axiom_test_support::WarpActionGenerator::generate(
            &state,
            &AdaptivePlayabilityPolicy.plan(&state),
            &OriginModel::default(),
            &PlannerContext::explicitly_unavailable(&state),
        );
        assert_eq!(
            generated.actions[0].node.resources,
            ResourceCost::new(
                crate::adaptive::REQUEST_SLICE_BYTES,
                crate::adaptive::REQUEST_SLICE_BYTES,
                0,
                1,
            )
        );
        assert_eq!(generated.actions.len(), 1);
    }
}

fn candidate(stage: HlsBootstrapStage) -> HlsCandidateSnapshot {
    HlsCandidateSnapshot {
        post: PostId::new("p0"),
        feed_offset: FeedOffset::new(0),
        view_probability: ViewProbability::new(1.0).expect("valid test fixture"),
        startup_value_ms: 2_000,
        cursor: Default::default(),
        player_preparation: Default::default(),
        state: HlsBootstrapState::Pending {
            stage,
            source: SOURCE.into(),
        },
    }
}
