mod gateway_fixture;

use core::time::Duration;
use gateway_fixture::progressive::progressive_harness;
use ghostr_gateway::progressive::capabilities::{
    ProgressiveCapabilities, ProgressiveCapabilityLimits,
};

#[tokio::test]
async fn oldest_capability_is_evicted_at_capacity() {
    let limits = ProgressiveCapabilityLimits::new(1, Duration::from_secs(60)).expect("limits");
    let capabilities = ProgressiveCapabilities::new(limits);
    let harness = progressive_harness("ghostr-capability-capacity");
    harness
        .bind_video("first", "https://cdn.example/first.mp4", Some(1))
        .await;
    harness
        .bind_video("second", "https://cdn.example/second.mp4", Some(1))
        .await;
    let first_snapshot = harness
        .store
        .media_snapshot("first")
        .await
        .expect("valid test fixture");
    let second_snapshot = harness
        .store
        .media_snapshot("second")
        .await
        .expect("valid test fixture");
    let first = capabilities
        .issue(&first_snapshot)
        .await
        .expect("valid test fixture");
    let second = capabilities
        .issue(&second_snapshot)
        .await
        .expect("valid test fixture");

    assert!(
        !capabilities
            .authorizes(first.as_str(), "first", &first_snapshot)
            .await
    );
    assert!(
        capabilities
            .authorizes(second.as_str(), "second", &second_snapshot)
            .await
    );
    assert_eq!(
        capabilities
            .issue(&second_snapshot)
            .await
            .expect("valid test fixture"),
        second
    );
    std::fs::remove_dir_all(harness.root).expect("remove store");
}
