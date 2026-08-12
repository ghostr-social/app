use ghostr_gateway::progressive::capabilities::{
    ProgressiveCapabilities, ProgressiveCapabilityLimits,
};
use std::time::Duration;

#[tokio::test(start_paused = true)]
async fn expired_capability_fails_closed() {
    let limits = ProgressiveCapabilityLimits::new(8, Duration::from_secs(5)).expect("limits");
    let capabilities = ProgressiveCapabilities::new(limits);
    let capability = capabilities.issue("clip").await;
    tokio::time::advance(Duration::from_secs(5)).await;

    assert!(!capabilities.authorizes(capability.as_str(), "clip").await);
}
