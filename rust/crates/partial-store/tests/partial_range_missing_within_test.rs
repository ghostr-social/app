mod store_fixture;

use std::sync::Arc;
use store_fixture::{plain_store, temp_root};
use tokio::sync::Mutex;

#[tokio::test]
async fn partial_range_missing_within_reports_only_the_gaps_inside_the_span() {
    let root = temp_root("ghostr-partial-missing");
    let store = plain_store(root.clone(), Arc::new(Mutex::new(0)));

    store.write_range("clip", 2, b"aa").await.expect("first");
    store.write_range("clip", 8, b"bb").await.expect("second");

    assert_eq!(
        store.missing_within("clip", 0..12).await.expect("gaps"),
        vec![0..2, 4..8, 10..12]
    );
    assert_eq!(
        store.missing_within("clip", 2..4).await.expect("covered"),
        Vec::<std::ops::Range<u64>>::new()
    );
    assert_eq!(
        store.missing_within("clip", 3..9).await.expect("clipped"),
        vec![4..8]
    );
    assert_eq!(
        store.missing_within("unknown", 0..4).await.expect("absent"),
        vec![0..4]
    );
    std::fs::remove_dir_all(root).expect("remove store");
}
