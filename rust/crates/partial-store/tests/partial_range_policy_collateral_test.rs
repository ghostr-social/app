use crate::partial_range_store::OutOfSpace;

#[tokio::test]
async fn policy_scratch_never_evicts_an_unselected_sibling() {
    let fixture = crate::tests::store_fixture::spaced_store(
        "policy-collateral",
        crate::tests::store_fixture::limits(20, 0),
        20,
    );
    fixture
        .store
        .write_range("sibling", 0, b"12345678")
        .await
        .expect("valid test fixture");
    fixture
        .store
        .set_total_len("sibling", 8)
        .await
        .expect("valid test fixture");
    fixture
        .store
        .write_range("clip", 0, b"abcdefghijkl")
        .await
        .expect("valid test fixture");
    fixture
        .store
        .set_total_len("clip", 12)
        .await
        .expect("valid test fixture");

    let result = fixture
        .store
        .evict_ranges("clip", core::slice::from_ref(&(4..8)))
        .await;

    assert!(result
        .expect_err("scenario must fail")
        .downcast_ref::<OutOfSpace>()
        .is_some());
    assert_eq!(fixture.store.used_bytes().await, 20);
    assert_eq!(
        fixture
            .store
            .present_ranges("sibling")
            .await
            .expect("valid test fixture"),
        vec![0..8]
    );
    assert_eq!(
        fixture
            .store
            .present_ranges("clip")
            .await
            .expect("valid test fixture"),
        vec![0..12]
    );
    crate::tests::store_fixture::discard(&fixture.root);
}
