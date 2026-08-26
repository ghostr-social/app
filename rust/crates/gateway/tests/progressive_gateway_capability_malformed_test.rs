mod gateway_fixture;

use gateway_fixture::progressive::progressive_harness;
use ghostr_gateway::progressive::capabilities::ProgressiveCapabilities;

#[tokio::test]
async fn malformed_capabilities_fail_closed_for_use_and_release() {
    let capabilities = ProgressiveCapabilities::production();
    let harness = progressive_harness("ghostr-capability-malformed");
    harness
        .bind_video("clip", "https://cdn.example/clip.mp4", Some(1))
        .await;
    let snapshot = harness
        .store
        .media_snapshot("clip")
        .await
        .expect("valid test fixture");

    assert!(!capabilities.recognizes("not-a-capability", "clip").await);
    assert!(
        !capabilities
            .authorizes("not-a-capability", "clip", &snapshot)
            .await
    );
    std::fs::remove_dir_all(harness.root).expect("remove store");
}
