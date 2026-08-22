mod store_fixture;

use ghostr_partial_store::partial_range_store::OutOfSpace;

#[tokio::test]
async fn policy_rewrite_never_spends_unadmitted_scratch_space() {
    let fixture =
        store_fixture::spaced_store("policy-peak-budget", store_fixture::limits(19, 0), 19);
    fixture
        .store
        .write_range("clip", 0, b"abcdefghijkl")
        .await
        .unwrap();
    fixture.store.set_total_len("clip", 12).await.unwrap();

    let result = fixture
        .store
        .evict_ranges("clip", std::slice::from_ref(&(4..8)))
        .await;

    assert!(result.unwrap_err().downcast_ref::<OutOfSpace>().is_some());
    assert_eq!(fixture.store.used_bytes().await, 12);
    assert_eq!(
        fixture.store.read_range("clip", 0..12).await.unwrap(),
        Some(b"abcdefghijkl".to_vec())
    );
    assert!(!fixture.root.join("clip.part.evict").exists());
    assert!(!fixture.root.join("clip.evict.intent").exists());
    store_fixture::discard(&fixture.root);
}
