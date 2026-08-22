use crate::tests::gateway_range_plan_fixture::demand_plan;
use ghostr_engine::ByteRange;

#[test]
fn conditional_promotion_excludes_sibling_ranges_until_response_semantics() {
    let demanded = ByteRange::new(120, 140);

    let work = demand_plan(demanded);
    let ranges: Vec<_> = work
        .transfers
        .iter()
        .map(|transfer| transfer.request.chunk.range)
        .collect();

    assert_eq!(ranges.first(), Some(&demanded), "{ranges:?}");
    assert_eq!(ranges, vec![demanded]);
}
