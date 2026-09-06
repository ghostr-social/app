use crate::tests::{busy_disk_fixture, store_fixture};
use core::time::Duration;
use std::sync::Arc;
use tokio::sync::Mutex;

#[test]
fn known_empty_candidate_remains_observable_while_disk_work_is_busy() {
    busy_disk_fixture::runtime().block_on(check_warm_snapshot());
}

async fn check_warm_snapshot() {
    let root = store_fixture::temp_root("warm-empty-snapshot");
    let store = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    let before = store
        .media_snapshot("candidate")
        .await
        .expect("initial observation");
    assert!(before.ranges().is_empty(), "candidate has no bytes yet");
    let disk = busy_disk_fixture::BusyDisk::occupy().await;
    let observed = tokio::time::timeout(
        Duration::from_millis(100),
        store.media_snapshot("candidate"),
    )
    .await;
    disk.finish().await;
    let after = observed
        .expect("known empty metadata must not wait for unrelated disk work")
        .expect("coherent warm observation");
    assert!(after.ranges().is_empty(), "empty candidate stays empty");
    assert_eq!(after.revision(), before.revision());
    store_fixture::discard(&root);
}
