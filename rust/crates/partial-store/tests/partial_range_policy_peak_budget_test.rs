use crate::partial_range_store::OutOfSpace;

#[tokio::test]
async fn policy_rewrite_never_spends_unadmitted_scratch_space() {
    let fixture = crate::tests::store_fixture::spaced_store(
        "policy-peak-budget",
        crate::tests::store_fixture::limits(19, 0),
        19,
    );
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
    assert_eq!(fixture.store.used_bytes().await, 12);
    assert_eq!(
        fixture
            .store
            .read_range("clip", 0..12)
            .await
            .expect("valid test fixture"),
        Some(b"abcdefghijkl".to_vec())
    );
    assert!(!fixture.root.join("clip.part.evict").exists());
    assert!(!fixture.root.join("clip.evict.intent").exists());
    crate::tests::store_fixture::discard(&fixture.root);
}
