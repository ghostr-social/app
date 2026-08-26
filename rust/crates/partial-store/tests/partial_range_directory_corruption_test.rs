#[tokio::test]
async fn a_directory_at_the_media_path_is_quarantined_without_releasing_usage() {
    let fixture = crate::tests::store_fixture::spaced_store(
        "directory-corruption",
        crate::tests::store_fixture::limits(16, 0),
        16,
    );
    fixture
        .store
        .write_range("clip", 0, b"abcdefgh")
        .await
        .expect("valid test fixture");
    let path = fixture.root.join("clip.part");
    std::fs::remove_file(&path).expect("valid test fixture");
    std::fs::create_dir(&path).expect("valid test fixture");

    assert!(fixture.store.read_range("clip", 0..8).await.is_err());
    assert!(fixture
        .store
        .present_ranges("clip")
        .await
        .expect("valid test fixture")
        .is_empty());
    assert_eq!(fixture.store.used_bytes().await, 8);
    assert!(path.is_dir());

    std::fs::remove_dir(&path).expect("valid test fixture");
    fixture.store.clear().await.expect("valid test fixture");
    assert_eq!(fixture.store.used_bytes().await, 0);
    fixture.store.clear().await.expect("valid test fixture");
    assert_eq!(fixture.store.used_bytes().await, 0);
    crate::tests::store_fixture::discard(&fixture.root);
}
