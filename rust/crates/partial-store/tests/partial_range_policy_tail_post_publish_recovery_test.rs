mod store_fixture;
mod tail_recovery_fixture;

#[tokio::test]
async fn published_tail_manifest_finishes_the_forward_recovery() {
    let root = tail_recovery_fixture::staged("tail-after-publish").await;
    tail_recovery_fixture::truncate(&root).await;
    tokio::fs::rename(
        root.join("clip.ranges.evict"),
        root.join("clip.ranges.json"),
    )
    .await
    .unwrap();

    let store = tail_recovery_fixture::reopen(&root).await;

    assert_eq!(store.used_bytes().await, 8);
    assert_eq!(store.present_ranges("clip").await.unwrap(), vec![0..8]);
    tail_recovery_fixture::assert_clean(&root);
    store_fixture::discard(&root);
}
