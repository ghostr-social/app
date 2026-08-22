mod gateway_fixture;

use gateway_fixture::progressive::progressive_harness;

#[tokio::test]
async fn unbound_store_bytes_cannot_become_a_progressive_asset() {
    let harness = progressive_harness("ghostr-progressive-unbound-asset");
    let snapshot = harness.store.media_snapshot("clip").await.unwrap();

    assert!(harness.capabilities.issue(&snapshot).await.is_err());
    std::fs::remove_dir_all(harness.root).ok();
}
