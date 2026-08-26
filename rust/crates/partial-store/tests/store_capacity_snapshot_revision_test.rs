use crate::tests::store_fixture::{discard, limits, spaced_store};

#[tokio::test]
async fn capacity_snapshot_revision_changes_when_equal_storage_totals_are_reconfigured() {
    let fixture = spaced_store("ghostr-cap-snapshot-revision", limits(800, 100), 1_000);
    fixture
        .store
        .write_range("first", 0, &[7; 200])
        .await
        .expect("initial write");
    let initial = fixture.store.capacity_snapshot().await;

    fixture
        .store
        .set_storage_budget(700)
        .await
        .expect("smaller budget");
    fixture
        .store
        .set_storage_budget(800)
        .await
        .expect("restored budget");
    let revised = fixture.store.capacity_snapshot().await;

    assert_eq!(initial.limit_bytes(), revised.limit_bytes());
    assert_eq!(initial.used_bytes(), revised.used_bytes());
    assert_ne!(initial.revision(), revised.revision());
    discard(&fixture.root);
}
