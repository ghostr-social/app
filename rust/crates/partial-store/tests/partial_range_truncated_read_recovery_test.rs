mod store_fixture;

#[tokio::test]
async fn a_stably_truncated_interval_is_discarded_and_requested_again() {
    let fixture =
        store_fixture::spaced_store("truncated-read-recovery", store_fixture::limits(64, 0), 64);
    fixture
        .store
        .write_range("clip", 0, b"abcdefgh")
        .await
        .unwrap();
    std::fs::OpenOptions::new()
        .write(true)
        .open(fixture.root.join("clip.part"))
        .unwrap()
        .set_len(4)
        .unwrap();

    assert_eq!(fixture.store.read_range("clip", 0..8).await.unwrap(), None);
    assert!(fixture
        .store
        .present_ranges("clip")
        .await
        .unwrap()
        .is_empty());
    assert_eq!(fixture.store.used_bytes().await, 0);
    assert!(!fixture.root.join("clip.part").exists());
    store_fixture::discard(&fixture.root);
}
