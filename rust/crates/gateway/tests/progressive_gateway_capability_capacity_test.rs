use ghostr_gateway::progressive::capabilities::{
    ProgressiveCapabilities, ProgressiveCapabilityLimits,
};
use std::time::Duration;

#[tokio::test]
async fn oldest_capability_is_evicted_at_capacity() {
    let limits = ProgressiveCapabilityLimits::new(1, Duration::from_secs(60)).expect("limits");
    let capabilities = ProgressiveCapabilities::new(limits);
    let first = capabilities.issue("first").await;
    let second = capabilities.issue("second").await;

    assert!(!capabilities.authorizes(first.as_str(), "first").await);
    assert!(capabilities.authorizes(second.as_str(), "second").await);
    assert_eq!(capabilities.issue("second").await, second);
}
