use crate::adaptive::{AdaptivePlayabilityPolicy, StorageSnapshot};
use crate::tests::adaptive_support::snapshot;
use crate::{ByteRange, PostId};

const STORE_BUDGET: u64 = 2_000_000;
const STORED_SIBLING: u64 = 1_000_000;

/// A full store must not stop the current video: the policy evicts the
/// lowest-value speculative ranges to make room instead of planning
/// writes the store will reject (which forfeits paid bytes wholesale).
#[test]
fn a_full_store_evicts_speculation_to_make_room_for_current_work() {
    let mut input = snapshot(3, 20_000_000, 1_000, 2);
    input.storage = StorageSnapshot::new(STORE_BUDGET, STORE_BUDGET);
    input.candidates[1].present = vec![ByteRange::new(0, STORED_SIBLING)];
    input.candidates[2].present = vec![ByteRange::new(0, STORED_SIBLING)];

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert!(
        !plan.evictions.is_empty(),
        "current work needs room from lowest-value speculation: {plan:#?}"
    );
    assert!(plan
        .evictions
        .iter()
        .all(|eviction| eviction.post != PostId::new("p0")));
    let planned: u64 = plan
        .allocations
        .iter()
        .filter(|work| work.post == PostId::new("p0"))
        .map(|work| work.request.reserved_network_bytes())
        .sum();
    let released: u64 = plan
        .evictions
        .iter()
        .map(|eviction| eviction.range.len())
        .sum();
    assert!(planned > 0, "{plan:#?}");
    assert!(
        planned <= input.storage.available_bytes() + released,
        "planned current bytes must fit the room actually made: \
         {planned} > {released}: {plan:#?}"
    );
}
