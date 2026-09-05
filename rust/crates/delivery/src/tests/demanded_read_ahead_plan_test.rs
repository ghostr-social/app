use crate::manager::reconcile_warp;
use crate::tests::gateway_range_plan_fixture::{demand_plan, sized_demand_plan, OBJECT_BYTES};
use ghostr_engine::adaptive::REQUEST_SLICE_BYTES;
use ghostr_engine::ByteRange;

/// A consumer blocked on a read keeps reading sequentially, so one blocked
/// read is planned as a single request slice from the blocked offset.
#[test]
fn blocked_read_is_planned_as_one_request_slice_from_its_offset() {
    let demanded = ByteRange::new(120, 140);
    let large = 4 * REQUEST_SLICE_BYTES;

    let ranges = executed_ranges(sized_demand_plan(demanded, large));

    assert_eq!(
        ranges,
        vec![ByteRange::new(120, 120 + REQUEST_SLICE_BYTES)],
        "{ranges:?}"
    );
}

#[test]
fn read_ahead_stops_at_the_end_of_the_object() {
    let demanded = ByteRange::new(120, 140);

    let ranges = executed_ranges(demand_plan(demanded));

    assert_eq!(
        ranges,
        vec![ByteRange::new(120, OBJECT_BYTES)],
        "{ranges:?}"
    );
}

fn executed_ranges(work: crate::manager::plan::PlannedWork) -> Vec<ByteRange> {
    reconcile_warp::execution(work)
        .transfers
        .iter()
        .map(|transfer| transfer.request.chunk.range)
        .collect()
}
