use crate::adaptive::{
    AdaptivePlayabilityPolicy, FeedOffset, HlsBootstrapStage, HlsBootstrapState,
    HlsCandidateSnapshot, PlannerCommand, PlannerContext, PlayerPreparation, ReserveCandidateKind,
    ReserveCandidateState, ViewProbability, WarpPlanner, WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;
use crate::PostId;

const SOURCE: &str = "https://hls.example/index.m3u8";

#[test]
fn exact_decoded_hls_is_the_first_ready_reserve() {
    let mut state = mixed(HlsBootstrapState::Ready, PlayerPreparation::PluginReady);
    let structural = AdaptivePlayabilityPolicy.plan(&state);
    let first = &structural.ready_reserve.candidates[0];
    assert_eq!(first.kind, ReserveCandidateKind::Hls);
    assert_eq!(first.state, ReserveCandidateState::HlsStructural);
    assert_eq!(structural.ready_reserve.ordered_ready(), 0);

    state.hls_candidates[0].player_preparation = PlayerPreparation::FirstFrameRendered;
    let ready = AdaptivePlayabilityPolicy.plan(&state);
    assert_eq!(
        ready.ready_reserve.candidates[0].kind,
        ReserveCandidateKind::Hls
    );
    assert_eq!(
        ready.ready_reserve.candidates[0].state,
        ReserveCandidateState::HlsReady
    );
    assert_eq!(ready.ready_reserve.ordered_ready(), 1);
}

#[test]
fn pending_nearest_hls_fences_and_wins_before_farther_progressive() {
    let mut state = mixed(
        HlsBootstrapState::Pending {
            stage: HlsBootstrapStage::RootManifest,
            source: SOURCE.to_owned(),
        },
        PlayerPreparation::Unverified,
    );
    state.network.connection_capacity = 1;
    state.network.connection_ceiling = 1;
    state.network.per_authority_request_limit = 1;
    let base = AdaptivePlayabilityPolicy.plan(&state);
    assert!(base
        .allocations
        .iter()
        .all(|work| work.post != PostId::new("p2")));
    assert_eq!(
        base.ready_reserve.candidates[0].state,
        ReserveCandidateState::HlsPending {
            stage: HlsBootstrapStage::RootManifest,
        }
    );
    let context = PlannerContext::explicitly_unavailable(&state)
        .with_segmented_storage_available_bytes(u64::MAX);
    let decision = WarpPlanner::default().plan(WarpPlannerInput::new(
        &state,
        &base,
        &OriginModel::default(),
        &context,
    ));
    assert!(matches!(
        decision.selected.map(|selected| selected.command),
        Some(PlannerCommand::FetchHlsBootstrap { post, .. }) if post == PostId::new("p1")
    ));
}

#[test]
fn current_playback_emergency_still_wins_before_hls_reserve() {
    let mut state = mixed(
        HlsBootstrapState::Pending {
            stage: HlsBootstrapStage::RootManifest,
            source: SOURCE.to_owned(),
        },
        PlayerPreparation::Unverified,
    );
    state.playback.buffer_ahead_ms = 0;
    let base = AdaptivePlayabilityPolicy.plan(&state);
    let context = PlannerContext::explicitly_unavailable(&state)
        .with_segmented_storage_available_bytes(u64::MAX);
    let decision = WarpPlanner::default().plan(WarpPlannerInput::new(
        &state,
        &base,
        &OriginModel::default(),
        &context,
    ));
    assert_eq!(
        decision.selected.expect("current rescue").node.post,
        PostId::new("p0")
    );
}

fn mixed(
    state: HlsBootstrapState,
    preparation: PlayerPreparation,
) -> crate::adaptive::PlayabilitySnapshot {
    let mut snapshot = snapshot(3, 20_000_000, 20_000, 0);
    snapshot.candidates.remove(1);
    snapshot.hls_candidates.push(HlsCandidateSnapshot {
        post: PostId::new("p1"),
        feed_offset: FeedOffset::new(1),
        view_probability: ViewProbability::new(0.88).expect("valid fixture"),
        startup_value_ms: 2_000,
        cursor: Default::default(),
        state,
        player_preparation: preparation,
    });
    snapshot
}
