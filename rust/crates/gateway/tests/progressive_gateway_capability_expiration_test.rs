mod gateway_fixture;

use gateway_fixture::progressive::progressive_harness;
use ghostr_gateway::progressive::capabilities::{
    ProgressiveCapabilities, ProgressiveCapabilityLimits,
};
use std::time::Duration;

#[tokio::test(start_paused = true)]
async fn expired_capability_fails_closed() {
    let limits = ProgressiveCapabilityLimits::new(8, Duration::from_secs(5)).expect("limits");
    let capabilities = ProgressiveCapabilities::new(limits);
    let harness = progressive_harness("ghostr-capability-expiration");
    harness
        .bind_video("clip", "https://cdn.example/clip.mp4", Some(1))
        .await;
    let snapshot = harness.store.media_snapshot("clip").await.unwrap();
    let capability = capabilities.issue(&snapshot).await.unwrap();
    tokio::time::advance(Duration::from_secs(5)).await;

    assert!(
        !capabilities
            .authorizes(capability.as_str(), "clip", &snapshot)
            .await
    );
    std::fs::remove_dir_all(harness.root).expect("remove store");
}
