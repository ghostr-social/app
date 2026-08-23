mod store_fixture;

#[tokio::test]
async fn resumable_takeover_fences_the_older_single_response() {
    let (root, store, transfer) = store_fixture::mode_fixture("resumable-takeover").await;
    store
        .begin_single_response(&transfer, 1, store_fixture::exact_response(8))
        .await
        .unwrap();
    let generation = store_fixture::source_generation();
    store
        .accept_generation(&transfer, generation.clone())
        .await
        .unwrap();

    assert!(!store
        .write_single_response_if_current(&transfer, 1, 0, b"stale")
        .await
        .unwrap());
    assert!(store
        .write_range_for_generation_if_current(&transfer, &generation, 0, b"fresh")
        .await
        .unwrap());
    assert_eq!(
        store.read_range("post", 0..5).await.unwrap(),
        Some(b"fresh".to_vec())
    );
    store_fixture::discard(&root);
}
