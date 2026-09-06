use crate::tests::{busy_disk_fixture, store_fixture};
use core::time::Duration;
use std::sync::Arc;
use tokio::sync::Mutex;

#[test]
fn known_partial_metadata_remains_observable_while_disk_work_is_busy() {
    busy_disk_fixture::runtime().block_on(check_warm_snapshot());
}

async fn check_warm_snapshot() {
    let root = store_fixture::temp_root("warm-partial-snapshot");
    let store = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    store.set_total_len("candidate", 8).await.expect("length");
    store
        .write_range("candidate", 0, b"data")
        .await
        .expect("partial bytes");
    let before = store
        .media_snapshot("candidate")
        .await
        .expect("initial observation");
    let disk = busy_disk_fixture::BusyDisk::occupy().await;
    let observed = tokio::time::timeout(
        Duration::from_millis(100),
        store.media_snapshot("candidate"),
    )
    .await;
    disk.finish().await;
    let after = observed
        .expect("known partial metadata must not wait for unrelated disk work")
        .expect("coherent warm observation");
    assert_eq!(after.ranges().len(), 1);
    assert_eq!(after.ranges()[0], 0..4);
    assert_eq!(after.total_len(), Some(8));
    assert_eq!(after.revision(), before.revision());
    store_fixture::discard(&root);
}
