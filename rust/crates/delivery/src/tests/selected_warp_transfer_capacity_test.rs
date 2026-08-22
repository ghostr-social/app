use crate::manager::concurrency::PlannedCapacity;

#[test]
fn a_selected_warp_transfer_opens_its_committed_request_lane() {
    let capacity = PlannedCapacity {
        total: 2,
        foreground_goal: 1,
    };

    assert_eq!(capacity.with_selected_transfer(2, 3, true).total, 3);
    assert_eq!(capacity.with_selected_transfer(2, 2, true).total, 2);
    assert_eq!(capacity.with_selected_transfer(2, 3, false).total, 2);
    assert_eq!(
        PlannedCapacity {
            total: 1,
            foreground_goal: 1,
        }
        .with_selected_transfer(2, 3, false)
        .total,
        2
    );
}
