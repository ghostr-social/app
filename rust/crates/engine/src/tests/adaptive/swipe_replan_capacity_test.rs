use crate::adaptive::{AdaptivePlayabilityPolicy, FeedOffset, InFlightAction};
use crate::tests::adaptive_support::snapshot;
use crate::{ByteRange, PostId};
use std::collections::HashSet;

#[test]
fn a_forward_swipe_retires_previous_work_outside_the_protected_window() {
    let mut input = snapshot(7, 20_000_000, 20_000, 6);
    input.playback.current = PostId::new("p3");
    input.network.connection_capacity = 4;
    input.network.connection_ceiling = 4;
    let navigation = input.navigation;
    for (index, candidate) in input.candidates.iter_mut().enumerate() {
        candidate.feed_offset = FeedOffset::new(index as i32 - 3);
        candidate.view_probability = navigation.view_probability(candidate.feed_offset);
    }
    for candidate in &mut input.candidates[..3] {
        candidate.in_flight.push(committed());
    }

    let plan = AdaptivePlayabilityPolicy.plan(&input);
    let planned: HashSet<_> = plan
        .allocations
        .iter()
        .map(|work| work.post.as_str())
        .collect();

    assert!(planned.contains("p4"));
    assert!(planned.contains("p5"));
    assert!(!planned.contains("p6"));
    assert!(plan.retained.is_empty(), "{plan:#?}");
}

fn committed() -> InFlightAction {
    InFlightAction::range(
        crate::ActionId::new(1),
        ByteRange::new(0, 250_000),
        "https://origin.example/media",
        20_000,
        true,
    )
}
