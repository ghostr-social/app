#![cfg(unix)]

mod cache_fixture;

use cache_fixture::video_cache_key;
use cache_fixture::{NativeCacheHarness, VIDEO_RESPONSE};
use std::collections::HashSet;
use std::os::unix::fs::PermissionsExt;

#[tokio::test]
async fn unremovable_invalid_active_blob_stays_charged_and_is_reported() {
    let harness = NativeCacheHarness::new("ghostr-cache-invalid-active");
    let key = video_cache_key();
    let cached = harness.download(&key, VIDEO_RESPONSE, None).await;
    tokio::fs::write(&cached.path, [])
        .await
        .expect("truncate blob");
    std::fs::set_permissions(&harness.directory, std::fs::Permissions::from_mode(0o500))
        .expect("block blob removal");

    let invalid = harness
        .cache
        .retain(&HashSet::from([key.clone()]))
        .await
        .expect("synchronize cache");

    std::fs::set_permissions(&harness.directory, std::fs::Permissions::from_mode(0o700))
        .expect("restore permissions");
    assert_eq!(invalid, HashSet::from([key]));
    assert!(cached.path.exists());
    assert_eq!(*harness.used_bytes.lock().await, 5);
    std::fs::remove_dir_all(harness.directory).expect("remove cache");
}
