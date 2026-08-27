use crate::manager::reconcile_warp;
use crate::tests::gateway_range_plan_fixture::demand_plan;
use ghostr_engine::ByteRange;

#[test]
fn gateway_demand_is_an_emergency_and_preserves_its_exact_range() {
    let demanded = ByteRange::new(9_000, 9_100);

    let work = demand_plan(demanded);

    assert!(work.emergency, "uncovered gateway demand is urgent");
    assert!(reconcile_warp::execution(work)
        .transfers
        .iter()
        .any(|transfer| transfer.request.chunk.range == demanded));
}
