mod store_fixture;

#[tokio::test]
async fn clear_attempts_every_key_and_keeps_failed_cleanup_charged() {
    let fixture = store_fixture::spaced_store("clear-failure", store_fixture::limits(8, 0), 8);
    fixture
        .store
        .write_range("blocked", 0, b"keep")
        .await
        .unwrap();
    fixture
        .store
        .write_range("later", 0, b"gone")
        .await
        .unwrap();
    std::fs::remove_file(fixture.root.join("blocked.ranges.json")).unwrap();
    std::fs::create_dir(fixture.root.join("blocked.ranges.json")).unwrap();

    fixture
        .store
        .clear()
        .await
        .expect_err("one key cannot clear");

    assert!(!fixture.root.join("later.part").exists());
    assert_eq!(fixture.store.used_bytes().await, 4);
    assert!(fixture
        .store
        .media_snapshot("blocked")
        .await
        .unwrap()
        .ranges()
        .is_empty());
    std::fs::remove_dir(fixture.root.join("blocked.ranges.json")).unwrap();
    fixture.store.clear().await.unwrap();
    assert_eq!(fixture.store.used_bytes().await, 0);
    store_fixture::discard(&fixture.root);
}
