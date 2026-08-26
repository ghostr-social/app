#[tokio::test]
async fn published_tail_manifest_finishes_the_forward_recovery() {
    let root = crate::tests::tail_recovery_fixture::staged("tail-after-publish").await;
    crate::tests::tail_recovery_fixture::truncate(&root).await;
    tokio::fs::rename(
        root.join("clip.ranges.evict"),
        root.join("clip.ranges.json"),
    )
    .await
    .expect("valid test fixture");

    let store = crate::tests::tail_recovery_fixture::reopen(&root).await;

    assert_eq!(store.used_bytes().await, 8);
    assert_eq!(
        store
            .present_ranges("clip")
            .await
            .expect("valid test fixture"),
        vec![0..8]
    );
    crate::tests::tail_recovery_fixture::assert_clean(&root);
    crate::tests::store_fixture::discard(&root);
}
