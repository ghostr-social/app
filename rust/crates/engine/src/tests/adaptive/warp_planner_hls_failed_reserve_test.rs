use crate::adaptive::{
    AdaptivePlayabilityPolicy, FeedOffset, HlsBootstrapState, HlsCandidateSnapshot,
    NextReserveEvidence, NextReserveInfeasibility, PlayerPreparation, ReserveCandidateKind,
    ReserveCandidateState, ViewProbability,
};
use crate::tests::adaptive_support::snapshot;
use crate::{ByteRange, PostId};

#[test]
fn decoder_failed_hls_is_infeasible_never_ready_or_protected() {
    let mut state = snapshot(2, 20_000_000, 20_000, 0);
    state.candidates[0].present = vec![ByteRange::new(0, 3_750_000)];
    state.candidates.remove(1);
    state.hls_candidates.push(HlsCandidateSnapshot {
        post: PostId::new("p1"),
        feed_offset: FeedOffset::new(1),
        view_probability: ViewProbability::new(0.88).expect("valid fixture"),
        startup_value_ms: 2_000,
        cursor: Default::default(),
        state: HlsBootstrapState::Ready,
        player_preparation: PlayerPreparation::Failed,
    });

    let plan = AdaptivePlayabilityPolicy.plan(&state);
    let candidate = &plan.ready_reserve.candidates[0];
    assert_eq!(candidate.kind, ReserveCandidateKind::Hls);
    assert_eq!(
        candidate.state,
        ReserveCandidateState::Infeasible {
            reason: NextReserveInfeasibility::PolicyDenied,
        }
    );
    assert_eq!(plan.ready_reserve.ordered_ready(), 0);
    assert_eq!(plan.ready_reserve.protected, 0);
    assert!(matches!(
        plan.next_reserve,
        NextReserveEvidence::Infeasible { post, .. } if post == PostId::new("p1")
    ));
}
