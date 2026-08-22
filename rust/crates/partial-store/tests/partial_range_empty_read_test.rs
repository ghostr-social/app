mod store_fixture;

#[tokio::test]
async fn an_empty_read_never_bypasses_checksum_verification() {
    let fixture = store_fixture::spaced_store("empty-read", store_fixture::limits(16, 0), 16);
    fixture
        .store
        .write_range("clip", 0, b"abcdefgh")
        .await
        .unwrap();

    assert_eq!(fixture.store.read_range("clip", 4..4).await.unwrap(), None);
    assert_eq!(
        fixture.store.present_ranges("clip").await.unwrap(),
        vec![0..8]
    );
    assert_eq!(fixture.store.used_bytes().await, 8);
    store_fixture::discard(&fixture.root);
}
