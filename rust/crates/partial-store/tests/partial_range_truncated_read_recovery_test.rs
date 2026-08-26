#[tokio::test]
async fn a_stably_truncated_interval_is_discarded_and_requested_again() {
    let fixture = crate::tests::store_fixture::spaced_store(
        "truncated-read-recovery",
        crate::tests::store_fixture::limits(64, 0),
        64,
    );
    fixture
        .store
        .write_range("clip", 0, b"abcdefgh")
        .await
        .expect("valid test fixture");
    std::fs::OpenOptions::new()
        .write(true)
        .open(fixture.root.join("clip.part"))
        .expect("valid test fixture")
        .set_len(4)
        .expect("valid test fixture");

    assert_eq!(
        fixture
            .store
            .read_range("clip", 0..8)
            .await
            .expect("valid test fixture"),
        None
    );
    assert!(fixture
        .store
        .present_ranges("clip")
        .await
        .expect("valid test fixture")
        .is_empty());
    assert_eq!(fixture.store.used_bytes().await, 0);
    assert!(!fixture.root.join("clip.part").exists());
    crate::tests::store_fixture::discard(&fixture.root);
}
