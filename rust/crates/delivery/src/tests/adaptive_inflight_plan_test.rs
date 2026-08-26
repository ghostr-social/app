use crate::tests::adaptive_plan_support::plan_with_active;
use ghostr_engine::{ActionId, ByteRange, PostId};

#[test]
fn manager_plan_subtracts_and_retains_exact_useful_inflight_ranges() {
    let active = ByteRange::new(0, 100_000);
    let work = plan_with_active(active, 5_000);
    let p1 = PostId::new("p1");

    assert!(work.transfers.iter().all(|transfer| {
        let range = transfer.request.chunk.range;
        transfer.request.chunk.post != p1 || range.end <= active.start || active.end <= range.start
    }));
    assert_eq!(work.retained, core::iter::once(ActionId::new(1)).collect());
    assert!(work.plan.retained.iter().any(|retained| {
        retained.post == p1
            && retained.request.requested_bytes() == active
            && retained.source == "https://media.example/p1.mp4"
    }));
}
