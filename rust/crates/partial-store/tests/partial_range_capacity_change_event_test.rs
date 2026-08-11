mod store_fixture;

use std::time::Duration;
use store_fixture::{discard, limits, paced_store};
use tokio::sync::watch;

#[tokio::test]
async fn only_real_capacity_changes_wake_parked_delivery() {
    let fixture = paced_store(
        "ghostr-capacity-event",
        limits(2_000, 0),
        1_000,
        Duration::from_secs(60),
    );
    fixture
        .store
        .write_range("held", 0, &[1; 400])
        .await
        .expect("seed held bytes");
    fixture.space.set(600);
    let lease = fixture.store.lease("held");
    let mut changes = fixture.store.capacity_changes();
    changes.borrow_and_update();

    fixture.store.recheck_capacity().await;
    assert!(!changes.has_changed().unwrap(), "same capacity stays quiet");

    fixture.space.set(900);
    fixture.store.recheck_capacity().await;
    take_change(&mut changes);

    drop(lease);
    take_change(&mut changes);

    fixture.store.set_storage_budget(2_500).await.unwrap();
    take_change(&mut changes);
    fixture.store.set_storage_budget(2_500).await.unwrap();
    assert!(!changes.has_changed().unwrap(), "same budget stays quiet");
    discard(&fixture.root);
}

fn take_change(changes: &mut watch::Receiver<u64>) {
    assert!(changes.has_changed().unwrap(), "capacity event");
    changes.borrow_and_update();
}
