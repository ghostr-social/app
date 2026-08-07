mod store_fixture;

use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use std::time::Duration;
use store_fixture::temp_root;
use tokio::sync::Mutex;
use tokio::time::timeout;

#[tokio::test]
async fn wakes_change_waiters_on_writes_and_total_length_declarations() {
    let root = temp_root("ghostr-partial-notify");
    let store = PartialRangeStore::new(root.clone(), Arc::new(Mutex::new(0)));
    let notify = store.change_notifier();

    let waiter = notify.notified();
    store
        .set_total_len("clip", 4)
        .await
        .expect("declare length");
    timeout(Duration::from_secs(1), waiter)
        .await
        .expect("woken by the total length declaration");
    assert_eq!(
        store.total_len("clip").await.expect("total length"),
        Some(4)
    );

    let waiter = notify.notified();
    store.write_range("clip", 0, b"ab").await.expect("write");
    timeout(Duration::from_secs(1), waiter)
        .await
        .expect("woken by the range write");
    std::fs::remove_dir_all(root).expect("remove store");
}
