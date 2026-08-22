mod store_fixture;
mod tail_recovery_fixture;

#[tokio::test]
async fn tail_intent_after_truncation_recovers_the_new_object() {
    let root = tail_recovery_fixture::staged("tail-after-truncate").await;
    tail_recovery_fixture::truncate(&root).await;

    let store = tail_recovery_fixture::reopen(&root).await;

    assert_eq!(store.used_bytes().await, 8);
    assert_eq!(store.present_ranges("clip").await.unwrap(), vec![0..8]);
    assert_eq!(
        store.read_range("clip", 0..8).await.unwrap(),
        Some(b"abcdefgh".to_vec())
    );
    tail_recovery_fixture::assert_clean(&root);
    store_fixture::discard(&root);
}
