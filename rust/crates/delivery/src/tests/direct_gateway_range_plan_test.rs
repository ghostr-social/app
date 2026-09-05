use crate::manager::reconcile_warp;
use crate::tests::gateway_range_plan_fixture::{demand_plan, OBJECT_BYTES};
use ghostr_engine::ByteRange;

#[test]
fn gateway_demand_is_an_emergency_with_bounded_read_ahead() {
    let demanded = ByteRange::new(9_000, 9_100);

    let work = demand_plan(demanded);

    assert!(work.emergency, "uncovered gateway demand is urgent");
    assert!(reconcile_warp::execution(work)
        .transfers
        .iter()
        .any(|transfer| transfer.request.chunk.range == ByteRange::new(9_000, OBJECT_BYTES)));
}
