mod store_fixture;

#[tokio::test]
async fn same_geometry_byte_change_has_a_distinct_snapshot_evidence_id() {
    let fixture =
        store_fixture::spaced_store("snapshot-evidence", store_fixture::limits(16, 0), 16);
    fixture
        .store
        .write_range("clip", 0, b"abcdefgh")
        .await
        .unwrap();
    let before = fixture.store.media_snapshot("clip").await.unwrap();
    let selected = 0..8;
    let before_id = before
        .evidence_id_for(std::slice::from_ref(&selected))
        .unwrap();

    fixture
        .store
        .write_range("clip", 0, b"ABCDEFGH")
        .await
        .unwrap();
    let after = fixture.store.media_snapshot("clip").await.unwrap();

    assert_eq!(after.ranges(), before.ranges());
    assert_eq!(after.revision(), before.revision());
    assert_ne!(
        after.evidence_id_for(std::slice::from_ref(&selected)),
        Some(before_id)
    );
    store_fixture::discard(&fixture.root);
}
