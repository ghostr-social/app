//! A restarted store owns the files the last run left behind. It has to
//! be able to give them back under pressure, or the first tight moment
//! refuses writes while the disk is full of videos the store does not
//! know it holds.

mod store_space;

use store_space::{discard, limits, reopened, spaced_store};

#[tokio::test]
async fn partial_range_reloaded_content_is_evictable() {
    let first = spaced_store("ghostr-reload-evict", limits(1_000_000, 1_000), 3_000);
    first
        .store
        .write_range("stale", 0, &[1; 400])
        .await
        .expect("an earlier run downloaded it");

    let restarted = reopened(&first);
    restarted.store.load_existing().await.expect("reload");
    assert_eq!(
        *restarted.used_bytes.lock().await,
        400,
        "the reload accounts for bytes this run never wrote"
    );

    restarted.space.set(600);
    assert_eq!(
        restarted.store.effective_capacity().await,
        0,
        "free space has fallen under the reserve"
    );
    assert_eq!(
        restarted.store.enforce_capacity().await,
        400,
        "the reloaded video is given back without ever being read"
    );
    assert!(!first.root.join("stale.part").exists(), "bytes on disk");
    assert!(!first.root.join("stale.ranges.json").exists(), "manifest");
    assert_eq!(*restarted.used_bytes.lock().await, 0);

    discard(&first.root);
}
