//! Verification provenance and byte accounting survive a restart: a
//! digest-checked file stays distinguishable from one that merely
//! finished downloading.

mod support;

use rust_lib_ghostr::video::partial_range_completion::Completion;
use rust_lib_ghostr::video::partial_range_store::PartialRangeStore;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use support::fixtures::temp_directory;
use tokio::sync::Mutex;

#[tokio::test]
async fn partial_range_completion_survives_a_restart() {
    let root = temp_directory("ghostr-partial-completion-reload");
    {
        let store = PartialRangeStore::new(root.clone(), Arc::new(Mutex::new(0)));
        let digest = format!("{:x}", Sha256::digest(b"headtail"));
        fill(&store, "checked").await;
        store
            .finalize("checked", Some(digest.as_str()))
            .await
            .expect("verified finalize");
        fill(&store, "plain").await;
        store.finalize("plain", None).await.expect("plain finalize");
    }

    let used_bytes = Arc::new(Mutex::new(0));
    let store = PartialRangeStore::new(root.clone(), used_bytes.clone());

    assert_eq!(
        store.completion("checked").await.expect("completion"),
        Some(Completion::Verified)
    );
    assert_eq!(
        store.completion("plain").await.expect("completion"),
        Some(Completion::Unverified)
    );
    assert!(store.is_complete("plain").await.expect("completeness"));
    assert_eq!(
        *used_bytes.lock().await,
        16,
        "both completed files stay accounted after the reload"
    );
    std::fs::remove_dir_all(root).expect("remove store");
}

async fn fill(store: &PartialRangeStore, key: &str) {
    store.write_range(key, 0, b"headtail").await.expect("bytes");
    store.set_total_len(key, 8).await.expect("total length");
}
