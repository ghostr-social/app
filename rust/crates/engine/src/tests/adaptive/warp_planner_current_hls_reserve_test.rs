use crate::adaptive::{
    AdaptivePlayabilityPolicy, ControlMode, FeedOffset, HlsBootstrapStage, HlsBootstrapState,
    HlsCandidateSnapshot, NextReserveInfeasibility, PlayerPreparation, ReserveCandidateState,
    ViewProbability,
};
use crate::tests::adaptive_support::snapshot;
use crate::PostId;

#[test]
fn failed_decoder_makes_current_hls_unsafe_and_blocks_future_reserve() {
    let mut state = snapshot(2, 20_000_000, 20_000, 120);
    state.candidates.remove(0);
    state.hls_candidates.push(hls(0, HlsBootstrapState::Ready, PlayerPreparation::Failed));

    let plan = AdaptivePlayabilityPolicy.plan(&state);

    assert_eq!(plan.mode, ControlMode::Emergency);
    assert_eq!(plan.ready_reserve.target, 1);
    assert_eq!(
        plan.ready_reserve.candidates[0].state,
        ReserveCandidateState::Infeasible {
            reason: NextReserveInfeasibility::CurrentUnprotected,
        },
    );
    assert!(plan.allocations.iter().all(|work| work.post != PostId::new("p1")));
}

#[test]
fn rendered_current_hls_is_safe_without_a_future_window() {
    let mut state = snapshot(1, 20_000_000, 20_000, 0);
    state.candidates.clear();
    state.hls_candidates.push(hls(
        0,
        HlsBootstrapState::Ready,
        PlayerPreparation::FirstFrameRendered,
    ));

    let plan = AdaptivePlayabilityPolicy.plan(&state);

    assert_eq!(plan.mode, ControlMode::Normal);
    assert_eq!(plan.ready_reserve.target, 0);
}

#[test]
fn rendered_current_hls_still_publishes_the_mixed_future_reserve() {
    let mut state = snapshot(3, 20_000_000, 20_000, 120);
    state.candidates.drain(0..2);
    state.hls_candidates.push(hls(
        0,
        HlsBootstrapState::Ready,
        PlayerPreparation::FirstFrameRendered,
    ));
    state.hls_candidates.push(hls(
        1,
        HlsBootstrapState::Pending {
            stage: HlsBootstrapStage::RootManifest,
            source: "https://hls.example/index.m3u8".to_owned(),
        },
        PlayerPreparation::Unverified,
    ));

    let plan = AdaptivePlayabilityPolicy.plan(&state);

    assert_eq!(plan.ready_reserve.target, 2);
    assert!(matches!(
        plan.ready_reserve.candidates[0].state,
        ReserveCandidateState::HlsPending { .. }
    ));
    assert!(plan.allocations.iter().all(|work| work.post != PostId::new("p2")));
}

fn hls(
    offset: i32,
    state: HlsBootstrapState,
    player_preparation: PlayerPreparation,
) -> HlsCandidateSnapshot {
    HlsCandidateSnapshot {
        post: PostId::new(format!("p{offset}")),
        feed_offset: FeedOffset::new(offset),
        view_probability: ViewProbability::new(0.88).expect("valid fixture"),
        startup_value_ms: 2_000,
        cursor: Default::default(),
        state,
        player_preparation,
    }
}
