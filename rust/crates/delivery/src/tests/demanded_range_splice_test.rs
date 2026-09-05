use crate::manager::reconcile_warp;
use crate::tests::gateway_range_plan_fixture::{demand_plan, OBJECT_BYTES};
use ghostr_engine::ByteRange;

#[test]
fn conditional_promotion_excludes_sibling_ranges_until_response_semantics() {
    let demanded = ByteRange::new(120, 140);

    let work = demand_plan(demanded);
    let ranges: Vec<_> = reconcile_warp::execution(work)
        .transfers
        .iter()
        .map(|transfer| transfer.request.chunk.range)
        .collect();

    assert_eq!(ranges, vec![ByteRange::new(demanded.start, OBJECT_BYTES)]);
}
