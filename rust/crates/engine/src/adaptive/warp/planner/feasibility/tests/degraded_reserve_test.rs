use super::{feasible, node};
use crate::adaptive::{ReserveDegradedReason, ResourceCost};

#[test]
fn infeasible_dependency_closure_enters_degraded_mode() {
    let path = [
        node(1, ResourceCost::new(1, 60, 0, 1), 0, &[]),
        node(2, ResourceCost::new(1, 60, 0, 1), 1_000, &[1]),
    ];
    let result = feasible(&path, ResourceCost::new(2, 100, 0, 2));

    assert!(result.reserve.degraded);
    assert_eq!(
        result.reserve.degraded_reason,
        Some(ReserveDegradedReason::NoFeasibleRescue)
    );
}
