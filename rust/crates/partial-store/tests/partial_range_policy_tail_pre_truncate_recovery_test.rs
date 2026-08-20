mod store_fixture;
mod tail_recovery_fixture;

#[tokio::test]
async fn tail_intent_before_truncation_recovers_the_old_object() {
    let root = tail_recovery_fixture::staged("tail-before-truncate").await;

    let store = tail_recovery_fixture::reopen(&root).await;

    assert_eq!(store.used_bytes().await, 12);
    assert_eq!(
        store.read_range("clip", 0..12).await.unwrap(),
        Some(b"abcdefghijkl".to_vec())
    );
    tail_recovery_fixture::assert_clean(&root);
    store_fixture::discard(&root);
}
