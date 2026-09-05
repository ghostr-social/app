use super::exploration_cost;

#[test]
fn cold_delivery_bytes_are_not_exploration_cost() {
    assert_eq!(exploration_cost(false, false, 9), (0, 0));
}

#[test]
fn admitted_exploration_charges_exact_received_bytes() {
    assert_eq!(exploration_cost(true, false, 9), (9, 0));
    assert_eq!(exploration_cost(true, true, 9), (9, 9));
}
