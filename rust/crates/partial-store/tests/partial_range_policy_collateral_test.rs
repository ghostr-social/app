mod store_fixture;

use ghostr_partial_store::partial_range_store::OutOfSpace;

#[tokio::test]
async fn policy_scratch_never_evicts_an_unselected_sibling() {
    let fixture =
        store_fixture::spaced_store("policy-collateral", store_fixture::limits(20, 0), 20);
    fixture
        .store
        .write_range("sibling", 0, b"12345678")
        .await
        .unwrap();
    fixture.store.set_total_len("sibling", 8).await.unwrap();
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
    assert_eq!(fixture.store.used_bytes().await, 20);
    assert_eq!(
        fixture.store.present_ranges("sibling").await.unwrap(),
        vec![0..8]
    );
    assert_eq!(
        fixture.store.present_ranges("clip").await.unwrap(),
        vec![0..12]
    );
    store_fixture::discard(&fixture.root);
}
