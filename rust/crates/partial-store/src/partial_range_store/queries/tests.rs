use super::read::ReadPlan;
use crate::partial_range_store::capacity::StoreCapacity;
use crate::partial_range_store::PartialRangeStore;
use std::io::{Seek, SeekFrom, Write};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

#[tokio::test]
async fn stale_corruption_result_does_not_condemn_repaired_bytes() {
    let root = std::env::temp_dir().join(format!(
        "ghostr-read-repair-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let store = PartialRangeStore::with_capacity(
        root.clone(),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    );
    store.write_range("clip", 0, b"abcdefgh").await.unwrap();
    corrupt(&root.join("clip.part"));

    let plan = capture(&store).await;
    let outcome = plan.execute().await.unwrap();
    assert!(!outcome.valid);
    store.write_range("clip", 0, b"abcdefgh").await.unwrap();
    assert_eq!(
        store.finish_read("clip", plan, Ok(outcome)).await.unwrap(),
        Some(b"abcdefgh".to_vec())
    );

    assert_eq!(
        store.read_range("clip", 0..8).await.unwrap(),
        Some(b"abcdefgh".to_vec())
    );
    assert_eq!(store.used_bytes().await, 8);
    std::fs::remove_dir_all(root).unwrap();
}

async fn capture(store: &PartialRangeStore) -> ReadPlan {
    let mut entries = store.entries.lock().await;
    let entry = store.entry(&mut entries, "clip").await.unwrap();
    ReadPlan::capture(&store.paths, "clip", entry, 0..8)
        .unwrap()
        .unwrap()
}

fn corrupt(path: &std::path::Path) {
    let mut file = std::fs::OpenOptions::new().write(true).open(path).unwrap();
    file.seek(SeekFrom::Start(3)).unwrap();
    file.write_all(b"X").unwrap();
    file.sync_all().unwrap();
}
