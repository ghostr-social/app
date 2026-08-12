mod store_fixture;

use store_fixture::{discard, limits, spaced_store};

#[tokio::test]
async fn capacity_snapshot_reports_live_effective_limit_and_total_usage() {
    let fixture = spaced_store("ghostr-cap-snapshot", limits(800, 100), 1_000);
    fixture
        .store
        .write_range("first", 0, &[7; 200])
        .await
        .expect("initial write");

    let initial = fixture.store.capacity_snapshot().await;
    assert_eq!(initial.limit_bytes(), 800);
    assert_eq!(initial.used_bytes(), 200);

    fixture.space.set(250);
    let constrained = fixture.store.capacity_snapshot().await;
    assert_eq!(constrained.limit_bytes(), 350);
    assert_eq!(constrained.used_bytes(), 200);
    discard(&fixture.root);
}
