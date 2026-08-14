mod gateway_fixture;

use gateway_fixture::progressive_delivery::ProgressiveDeliveryHarness;

#[tokio::test]
async fn progressive_journey_harness_removes_its_store() {
    let harness = ProgressiveDeliveryHarness::start("ghostr-progressive-cleanup");
    let root = harness.delivery.root.clone();
    std::fs::create_dir_all(&root).expect("materialize journey store");

    drop(harness);

    assert!(!root.exists(), "journey fixture store leaked: {root:?}");
}
