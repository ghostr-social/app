use super::first_deficit;
use crate::adaptive::{
    AdaptivePlayabilityPolicy, FeedOffset, HlsBootstrapState, HlsCandidateSnapshot,
    PlayerPreparation, ReserveCandidateState, ViewProbability,
};
use crate::tests::adaptive_support::snapshot;
use crate::{ByteRange, PostId};

#[test]
fn decoded_hls_prefix_advances_to_the_later_progressive_deficit() {
    let mut state = snapshot(3, 20_000_000, 20_000, 120);
    state.candidates[0].present = vec![ByteRange::new(0, 3_750_000)];
    state.candidates.remove(1);
    state.hls_candidates.push(HlsCandidateSnapshot {
        post: PostId::new("p1"),
        feed_offset: FeedOffset::new(1),
        view_probability: ViewProbability::new(0.88).expect("valid fixture"),
        startup_value_ms: 2_000,
        cursor: Default::default(),
        state: HlsBootstrapState::Ready,
        player_preparation: PlayerPreparation::FirstFrameRendered,
    });
    let base = AdaptivePlayabilityPolicy.plan(&state);

    assert_eq!(base.ready_reserve.target, 2);
    assert_eq!(
        base.ready_reserve.candidates[0].state,
        ReserveCandidateState::HlsReady
    );
    assert_eq!(
        first_deficit(&state, &base).map(|candidate| &candidate.post),
        Some(&PostId::new("p2")),
    );
}
