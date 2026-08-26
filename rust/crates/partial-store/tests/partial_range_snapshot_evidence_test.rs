#[tokio::test]
async fn same_geometry_byte_change_has_a_distinct_snapshot_evidence_id() {
    let fixture = crate::tests::store_fixture::spaced_store(
        "snapshot-evidence",
        crate::tests::store_fixture::limits(16, 0),
        16,
    );
    fixture
        .store
        .write_range("clip", 0, b"abcdefgh")
        .await
        .expect("valid test fixture");
    let before = fixture
        .store
        .media_snapshot("clip")
        .await
        .expect("valid test fixture");
    let selected = 0..8;
    let before_id = before
        .evidence_id_for(core::slice::from_ref(&selected))
        .expect("valid test fixture");

    fixture
        .store
        .write_range("clip", 0, b"ABCDEFGH")
        .await
        .expect("valid test fixture");
    let after = fixture
        .store
        .media_snapshot("clip")
        .await
        .expect("valid test fixture");

    assert_eq!(after.ranges(), before.ranges());
    assert_eq!(after.revision(), before.revision());
    assert_ne!(
        after.evidence_id_for(core::slice::from_ref(&selected)),
        Some(before_id)
    );
    crate::tests::store_fixture::discard(&fixture.root);
}
