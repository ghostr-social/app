use crate::adaptive::axiom_test_support::WarpActionGenerator;
use crate::adaptive::{
    AdaptivePlayabilityPolicy, FeedOffset, HlsBootstrapStage, HlsBootstrapState,
    HlsCandidateSnapshot, HlsObjectCursor, HlsTransport, PlannerCommand, PlannerContext,
    ResourceCost, ViewProbability,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;
use crate::PostId;

#[test]
fn known_hls_tail_has_exact_cursor_and_incremental_resource_cost() {
    let mut state = snapshot(0, 80_000_000, 0, 0);
    let cursor = HlsObjectCursor::new(7, 256 * 1024, Some(300 * 1024), HlsTransport::ResumeRange);
    state.hls_candidates.push(candidate(cursor));

    let generated = WarpActionGenerator::generate(
        &state,
        &AdaptivePlayabilityPolicy.plan(&state),
        &OriginModel::default(),
        &PlannerContext::explicitly_unavailable(&state),
    );

    let selected = generated.actions.first().expect("terminal HLS tail");
    assert_eq!(
        selected.node.resources,
        ResourceCost::new(44 * 1024, 44 * 1024, 0, 1)
    );
    assert!(matches!(
        selected.command,
        PlannerCommand::FetchHlsBootstrap {
            cursor: observed,
            maximum_bytes: 45_056,
            ..
        } if observed == cursor
    ));
}

#[test]
fn cursor_rejects_an_object_larger_than_the_stage_envelope() {
    let cursor = HlsObjectCursor::new(
        8,
        0,
        Some(HlsBootstrapStage::RootManifest.maximum_bytes() + 1),
        HlsTransport::Start,
    );

    assert_eq!(
        cursor.block_bytes(HlsBootstrapStage::RootManifest, 256 * 1024),
        None
    );
}

#[test]
fn ordinary_hls_blocks_enforce_the_paper_size_bounds() {
    for stage in [
        HlsBootstrapStage::RootManifest,
        HlsBootstrapStage::FirstSegment,
    ] {
        assert_eq!(stage.block_bytes(64 * 1024), 128 * 1024);
        assert_eq!(stage.block_bytes(256 * 1024), 256 * 1024);
        assert_eq!(stage.block_bytes(1024 * 1024), 512 * 1024);
    }
}

fn candidate(cursor: HlsObjectCursor) -> HlsCandidateSnapshot {
    HlsCandidateSnapshot {
        post: PostId::new("p0"),
        feed_offset: FeedOffset::new(0),
        view_probability: ViewProbability::new(1.0).expect("valid test fixture"),
        startup_value_ms: 2_000,
        cursor,
        state: HlsBootstrapState::Pending {
            stage: HlsBootstrapStage::Initialization,
            source: "https://hls.example/init.mp4".into(),
        },
    }
}
