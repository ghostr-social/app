use crate::tests::adaptive_plan_assertions::allocated_posts;
use crate::tests::adaptive_plan_support::{plan, plan_with_present};
use ghostr_engine::adaptive::StorageSnapshot;
use ghostr_engine::{ByteRange, PostId};
use std::collections::HashMap;

#[test]
fn live_storage_pressure_contracts_manager_origin_admission() {
    let roomy = plan(20_000, 4_000_000, StorageSnapshot::new(2_000_000_000, 0));
    let pressured = plan(20_000, 4_000_000, StorageSnapshot::new(1_000_000, 950_000));

    assert!(allocated_posts(&roomy).len() > 1);
    assert_eq!(allocated_posts(&pressured).len(), 1);
}

#[test]
fn manager_work_carries_the_lowest_value_exact_eviction() {
    let present = HashMap::from([
        (PostId::new("p1"), vec![ByteRange::new(0, 250_000)]),
        (PostId::new("p8"), vec![ByteRange::new(0, 250_000)]),
    ]);
    let work = plan_with_present(StorageSnapshot::new(1_000_000, 1_240_000), present);

    assert_eq!(work.evictions.len(), 1, "{:#?}", work.evictions);
    assert_eq!(work.evictions[0].post, PostId::new("p8"));
    assert_eq!(work.evictions[0].range, ByteRange::new(0, 250_000));
}
