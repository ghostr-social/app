use super::scheduler_support::context;
use crate::discovery::scheduler_retry::retry_delay;
use std::time::Duration;

#[test]
fn retry_ladder_caps_at_eight_seconds_and_spreads_contexts() {
    let main = context("main");
    let following = context("following");
    let first = retry_delay(&main, 0);
    let stagger = first - Duration::from_millis(500);

    assert!(stagger >= Duration::from_millis(25));
    assert!(stagger <= Duration::from_millis(200));
    assert_ne!(first, retry_delay(&following, 0));
    assert_eq!(retry_delay(&main, 1), Duration::from_secs(1) + stagger);
    assert_eq!(retry_delay(&main, 2), Duration::from_secs(2) + stagger);
    assert_eq!(retry_delay(&main, 3), Duration::from_secs(4) + stagger);
    assert_eq!(retry_delay(&main, 4), Duration::from_secs(8) + stagger);
    assert_eq!(retry_delay(&main, 20), Duration::from_secs(8) + stagger);
}
