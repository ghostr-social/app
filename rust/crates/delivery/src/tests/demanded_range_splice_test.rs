use crate::tests::gateway_range_plan_fixture::demand_plan;
use ghostr_engine::ByteRange;

#[test]
fn exact_demand_preserves_the_surrounding_playable_extent() {
    let demanded = ByteRange::new(120, 140);

    let work = demand_plan(demanded);
    let ranges: Vec<_> = work
        .transfers
        .iter()
        .map(|transfer| transfer.request.chunk.range)
        .collect();

    assert_eq!(ranges.first(), Some(&demanded), "{ranges:?}");
    assert!(ranges.contains(&ByteRange::new(100, 120)), "{ranges:?}");
    assert!(ranges.contains(&ByteRange::new(140, 200)), "{ranges:?}");
}
