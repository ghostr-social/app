use super::support::planned_transfer;
use crate::manager::concurrency::planned_capacity;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::PostId;
use std::collections::HashSet;

#[test]
fn multiple_paid_ranges_for_one_video_reserve_one_lane() {
    let transfers = [planned_transfer(
        "next",
        "same",
        PreemptionAuthority::Transition,
    )];
    let retained = [PostId::new("current"), PostId::new("current")]
        .into_iter()
        .collect::<HashSet<_>>();

    assert_eq!(planned_capacity(1, 3, &transfers, &retained).total, 2);
}
