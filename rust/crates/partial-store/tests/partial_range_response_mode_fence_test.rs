#[tokio::test]
async fn resumable_takeover_fences_the_older_single_response() {
    let (root, store, transfer) =
        crate::tests::store_fixture::mode_fixture("resumable-takeover").await;
    store
        .begin_single_response(&transfer, 1, crate::tests::store_fixture::exact_response(8))
        .await
        .expect("valid test fixture");
    let generation = crate::tests::store_fixture::source_generation();
    store
        .accept_generation(&transfer, generation.clone())
        .await
        .expect("valid test fixture");

    assert!(!store
        .write_single_response_if_current(&transfer, 1, 0, b"stale")
        .await
        .expect("valid test fixture"));
    assert!(store
        .write_range_for_generation_if_current(&transfer, &generation, 0, b"fresh")
        .await
        .expect("valid test fixture"));
    assert_eq!(
        store
            .read_range("post", 0..5)
            .await
            .expect("valid test fixture"),
        Some(b"fresh".to_vec())
    );
    crate::tests::store_fixture::discard(&root);
}
