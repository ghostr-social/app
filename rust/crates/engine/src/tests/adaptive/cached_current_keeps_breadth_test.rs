use crate::adaptive::AdaptivePlayabilityPolicy;
use crate::playback::PlaybackPhase;
use crate::tests::adaptive_support::{frontier, snapshot};
use crate::{ByteRange, PostId};

fn fully_cached() -> ByteRange {
    ByteRange::new(0, 3_750_000)
}

/// The last seconds of a fully cached video are not a delivery
/// emergency: no network work can improve the current post, so the
/// policy must keep preparing the upcoming candidates instead of
/// collapsing to a single shallow transition.
#[test]
fn low_buffer_on_a_fully_cached_current_keeps_upcoming_breadth() {
    let mut input = snapshot(4, 20_000_000, 2_000, 2);
    input.candidates[0].present = vec![fully_cached()];

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    let posts = frontier(&plan);
    assert!(posts.contains(&PostId::new("p1")), "{plan:#?}");
    assert!(
        posts.contains(&PostId::new("p2")),
        "swiping at the end of a cached video must find prepared candidates: {plan:#?}"
    );
}

#[test]
fn startup_phase_on_a_fully_cached_current_keeps_upcoming_breadth() {
    let mut input = snapshot(4, 20_000_000, 0, 2);
    input.playback.phase = PlaybackPhase::Starting;
    input.candidates[0].present = vec![fully_cached()];

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    let posts = frontier(&plan);
    assert!(posts.contains(&PostId::new("p1")), "{plan:#?}");
    assert!(posts.contains(&PostId::new("p2")), "{plan:#?}");
}
