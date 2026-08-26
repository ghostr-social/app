#[tokio::test]
async fn tail_intent_after_truncation_recovers_the_new_object() {
    let root = crate::tests::tail_recovery_fixture::staged("tail-after-truncate").await;
    crate::tests::tail_recovery_fixture::truncate(&root).await;

    let store = crate::tests::tail_recovery_fixture::reopen(&root).await;

    assert_eq!(store.used_bytes().await, 8);
    assert_eq!(
        store
            .present_ranges("clip")
            .await
            .expect("valid test fixture"),
        vec![0..8]
    );
    assert_eq!(
        store
            .read_range("clip", 0..8)
            .await
            .expect("valid test fixture"),
        Some(b"abcdefgh".to_vec())
    );
    crate::tests::tail_recovery_fixture::assert_clean(&root);
    crate::tests::store_fixture::discard(&root);
}
