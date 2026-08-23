mod store_fixture;

#[tokio::test]
async fn staged_single_response_preserves_sparse_bytes_until_atomic_takeover() {
    let (root, store, transfer) = store_fixture::mode_fixture("single-takeover").await;
    let generation = store_fixture::source_generation();
    store
        .accept_generation(&transfer, generation.clone())
        .await
        .unwrap();
    store
        .begin_single_response(&transfer, 2, store_fixture::exact_response(8))
        .await
        .unwrap();
    store
        .write_range_for_generation_if_current(&transfer, &generation, 0, b"old data")
        .await
        .unwrap();
    store
        .write_single_response_if_current(&transfer, 2, 0, b"new data")
        .await
        .unwrap();
    assert_eq!(
        store.read_range("post", 0..8).await.unwrap(),
        Some(b"old data".to_vec())
    );
    store
        .finish_single_response(&transfer, 2, Some(8), true)
        .await
        .unwrap();
    assert_eq!(
        store.read_range("post", 0..8).await.unwrap(),
        Some(b"new data".to_vec())
    );
    assert!(!store
        .write_range_for_generation_if_current(&transfer, &generation, 0, b"late")
        .await
        .unwrap());
    store_fixture::discard(&root);
}
