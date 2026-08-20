mod store_fixture;

#[tokio::test]
async fn a_directory_at_the_media_path_is_quarantined_without_releasing_usage() {
    let fixture =
        store_fixture::spaced_store("directory-corruption", store_fixture::limits(16, 0), 16);
    fixture
        .store
        .write_range("clip", 0, b"abcdefgh")
        .await
        .unwrap();
    let path = fixture.root.join("clip.part");
    std::fs::remove_file(&path).unwrap();
    std::fs::create_dir(&path).unwrap();

    assert!(fixture.store.read_range("clip", 0..8).await.is_err());
    assert!(fixture
        .store
        .present_ranges("clip")
        .await
        .unwrap()
        .is_empty());
    assert_eq!(fixture.store.used_bytes().await, 8);
    assert!(path.is_dir());

    std::fs::remove_dir(&path).unwrap();
    fixture.store.clear().await.unwrap();
    assert_eq!(fixture.store.used_bytes().await, 0);
    fixture.store.clear().await.unwrap();
    assert_eq!(fixture.store.used_bytes().await, 0);
    store_fixture::discard(&fixture.root);
}
