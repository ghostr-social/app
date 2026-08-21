use super::{feasible, node};
use crate::adaptive::ResourceCost;

#[test]
fn local_rescue_reports_zero_reserved_request_slots() {
    let limits = ResourceCost::new(0, 16, 4, 0);
    let local = ResourceCost::new(0, 15, 4, 0);
    let result = feasible(&[node(1, local, 1_000, &[])], limits);

    assert!(!result.reserve.degraded);
    assert_eq!(result.reserve.reserved_request_slots, 0);
    assert_eq!(result.reserve.reserved_network_bytes, 0);
    assert_eq!(result.nodes.len(), 1);
}
