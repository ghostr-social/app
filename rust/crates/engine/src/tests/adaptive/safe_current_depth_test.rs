use crate::adaptive::AdaptivePlayabilityPolicy;
use crate::tests::adaptive_support::{planned_playable_ms, snapshot};
use crate::PostId;

/// Safe-mode refills must aim well above the emergency threshold, or
/// steady playback oscillates between safe trickles and emergency
/// cancel storms around the 4-second line.
#[test]
fn safe_current_refill_clears_the_emergency_threshold_with_margin() {
    let input = snapshot(1, 20_000_000, 4_500, 2);

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert!(
        planned_playable_ms(&plan, &PostId::new("p0")) >= 1_500,
        "buffer plus refill must clear 4s with margin: {plan:#?}"
    );
}

/// A comfortably buffered current video needs no further reserve; the
/// budget belongs to upcoming candidates instead.
#[test]
fn a_comfortably_buffered_current_video_plans_no_further_depth() {
    let input = snapshot(1, 20_000_000, 20_000, 2);

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert_eq!(planned_playable_ms(&plan, &PostId::new("p0")), 0, "{plan:#?}");
}
