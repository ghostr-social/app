//! Device pass 3 measured the progressive store at 8 KB after every
//! launch. A restart must find what the last run left behind: completed
//! and partial files with a usable manifest are accounted for and
//! reusable before anything asks for them.

mod support;

use rust_lib_ghostr::video::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use support::fixtures::temp_directory;
use tokio::sync::Mutex;

#[tokio::test]
async fn partial_range_store_reloads_its_contents_at_startup() {
    let root = temp_directory("ghostr-store-reload");
    seed(&root).await;
    std::fs::write(root.join("orphan.part"), b"nomanifest").expect("orphan bytes");

    let used_bytes = Arc::new(Mutex::new(0));
    let store = PartialRangeStore::new(root.clone(), used_bytes.clone());
    store.load_existing().await.expect("reload");

    assert_eq!(
        *used_bytes.lock().await,
        12,
        "eight completed bytes plus four partial ones, before anything is read"
    );
    assert!(store.is_complete("done").await.expect("completed video"));
    assert_eq!(
        store
            .read_range("done", 0..8)
            .await
            .expect("completed read"),
        Some(b"headtail".to_vec()),
        "the finished file is reused, not re-downloaded"
    );
    assert_eq!(
        store.present_ranges("half").await.expect("ranges"),
        vec![0..4]
    );
    assert_eq!(
        store.missing_within("half", 0..8).await.expect("missing"),
        vec![4..8],
        "only the tail is still owed"
    );
    assert!(
        !root.join("orphan.part").exists(),
        "partial bytes with no manifest cannot be resumed and must not leak"
    );
    assert_eq!(
        *used_bytes.lock().await,
        12,
        "reading does not double-count"
    );
    store.load_existing().await.expect("idempotent reload");
    assert_eq!(*used_bytes.lock().await, 12, "reload does not double-count");
    std::fs::remove_dir_all(root).expect("remove store");
}

async fn seed(root: &std::path::Path) {
    let store = PartialRangeStore::new(root.to_path_buf(), Arc::new(Mutex::new(0)));
    store
        .write_range("done", 0, b"headtail")
        .await
        .expect("bytes");
    store.set_total_len("done", 8).await.expect("total length");
    store.finalize("done", None).await.expect("finalize");
    store.write_range("half", 0, b"head").await.expect("bytes");
    store.set_total_len("half", 8).await.expect("total length");
}
