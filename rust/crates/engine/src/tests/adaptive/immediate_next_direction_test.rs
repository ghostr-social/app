use crate::adaptive::{AdaptivePlayabilityPolicy, FeedOffset, NextReserveEvidence};
use crate::playback::PlaybackPhase;
use crate::tests::adaptive_support::snapshot;
use crate::PostId;

#[test]
fn immediate_next_is_the_forward_sibling_after_a_swipe() {
    let mut input = snapshot(3, 700_000, 0, 2);
    input.playback.current = PostId::new("p1");
    input.playback.phase = PlaybackPhase::Starting;
    input.network.connection_capacity = 1;
    input.network.connection_ceiling = 1;
    input.candidates[0].feed_offset = FeedOffset::new(-1);
    input.candidates[1].feed_offset = FeedOffset::new(0);
    input.candidates[2].feed_offset = FeedOffset::new(1);

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert!(matches!(
        plan.next_reserve,
        NextReserveEvidence::Granted { post, .. } if post == PostId::new("p2")
    ));
}
