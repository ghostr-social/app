use crate::adaptive::{AdaptivePlayabilityPolicy, MediaLayout, PlayableRange};
use crate::tests::adaptive_support::{frontier, snapshot};
use crate::tests::support::set_reliable_total_bytes;
use crate::{ByteRange, PostId};

#[test]
fn cached_upcoming_playability_can_pay_a_complete_file_delivery_cost() {
    let mut input = snapshot(3, 50_000_000, 0, 2);
    let observed_at_ms = input.observed_at_ms;
    input.candidates[1].present = input.candidates[1]
        .playable_ranges
        .iter()
        .map(|playable| playable.bytes)
        .collect();
    input.candidates[2].layout = MediaLayout::RequiresCompleteFile;
    set_reliable_total_bytes(&mut input.candidates[2], 20_000_000, observed_at_ms);
    input.candidates[2].playable_ranges = vec![PlayableRange {
        bytes: ByteRange::new(0, 20_000_000),
        playable_ms: 60_000,
    }];

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert!(frontier(&plan).contains(&PostId::new("p2")), "{plan:#?}");
}
