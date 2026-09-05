use super::{NetworkTokenBucket, WarpPlanner};

#[test]
fn successive_body_windows_share_the_planners_refillable_allowance() {
    let mut planner = WarpPlanner::default();
    planner.network = Some(NetworkTokenBucket::new(100, 100, 0));
    assert!(planner.reserve_network_window(80, 0));
    assert!(!planner.reserve_network_window(80, 0));
    assert_eq!(planner.network_window_deadline_ms(80, 0), Some(600));
    assert!(planner.reserve_network_window(80, 600));
    assert!(!planner.reserve_network_window(41, 800));
    assert_eq!(planner.network_tokens(800), 20);
}
