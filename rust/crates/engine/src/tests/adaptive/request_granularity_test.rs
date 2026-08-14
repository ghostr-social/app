use crate::adaptive::{AdaptivePlayabilityPolicy, PlayableRange, REQUEST_SLICE_BYTES};
use crate::tests::adaptive_support::snapshot;
use crate::ByteRange;

const WHOLE_FILE_BYTES: u64 = 2 * 1024 * 1024;
const WHOLE_FILE_MS: u64 = 6_000;
const MICRO_EXTENT_BYTES: u64 = 10_000;
const MICRO_EXTENT_COUNT: u64 = 200;
const MICRO_EXTENT_MS: u64 = 30;

#[test]
fn an_unparsed_whole_file_is_requested_in_bounded_slices() {
    let mut input = snapshot(1, 20_000_000, 500, 2);
    input.candidates[0].playable_ranges = vec![PlayableRange {
        bytes: ByteRange::new(0, WHOLE_FILE_BYTES),
        playable_ms: WHOLE_FILE_MS,
    }];

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert!(!plan.allocations.is_empty(), "{plan:#?}");
    assert!(
        plan.allocations
            .iter()
            .all(|work| work.range.len() <= REQUEST_SLICE_BYTES),
        "an interrupted oversized request loses all its undelivered paid bytes: {plan:#?}"
    );
    assert!(plan.allocations.len() > 1, "{plan:#?}");
}

#[test]
fn adjacent_micro_extents_coalesce_into_larger_requests() {
    let mut input = snapshot(1, 20_000_000, 4_500, 2);
    input.candidates[0].playable_ranges = (0..MICRO_EXTENT_COUNT)
        .map(|index| PlayableRange {
            bytes: ByteRange::new(index * MICRO_EXTENT_BYTES, (index + 1) * MICRO_EXTENT_BYTES),
            playable_ms: MICRO_EXTENT_MS,
        })
        .collect();

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    let ceiling = (MICRO_EXTENT_COUNT * MICRO_EXTENT_BYTES).div_ceil(REQUEST_SLICE_BYTES) + 1;
    assert!(!plan.allocations.is_empty(), "{plan:#?}");
    assert!(
        (plan.allocations.len() as u64) <= ceiling,
        "each request costs a round trip; micro-requests crawl on real networks: \
         {} allocations",
        plan.allocations.len()
    );
    assert!(plan
        .allocations
        .iter()
        .all(|work| work.range.len() <= REQUEST_SLICE_BYTES));
}

#[test]
fn coalescing_does_not_bridge_gaps_between_extents() {
    let mut input = snapshot(1, 20_000_000, 500, 2);
    input.candidates[0].playable_ranges = vec![
        PlayableRange {
            bytes: ByteRange::new(0, MICRO_EXTENT_BYTES),
            playable_ms: MICRO_EXTENT_MS,
        },
        PlayableRange {
            bytes: ByteRange::new(2 * MICRO_EXTENT_BYTES, 3 * MICRO_EXTENT_BYTES),
            playable_ms: MICRO_EXTENT_MS,
        },
    ];

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert!(
        plan.allocations.iter().all(|work| {
            work.range.end <= MICRO_EXTENT_BYTES || work.range.start >= 2 * MICRO_EXTENT_BYTES
        }),
        "a request must never pay for bytes outside the wanted extents: {plan:#?}"
    );
}
