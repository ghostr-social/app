mod delivery_fixture;
mod range_fixture;

use core::time::Duration;
use delivery_fixture::items::{focus_now, seed_range, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_delivery::cache_registry::CacheStatus;

#[tokio::test]
async fn all_retired_progressive_item_remains_failed_in_cache_registry() {
    let origin = range_fixture::ranged::serve_ranged(vec![7; 16])
        .await
        .replace("/video.mp4", "/gone.mp4");
    let mut options = DeliveryOptions::default();
    options.tuning.retry.permanent_attempts = 1;
    let harness = start_harness("ghostr-progressive-terminal-cache", options);
    let item = sized_item("failed", &origin, 16, 1_000);
    seed_range(&harness.store, &item, 0, &[7]).await;

    harness.handle.update_focus(focus_now(vec![item], 0, 0));

    let failed = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            if let Some(video) = harness
                .cache
                .videos()
                .into_iter()
                .find(|video| video.id == "failed" && video.status == CacheStatus::Failed)
            {
                break video;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("all-retired item remained in the cache registry");

    assert_eq!(failed.status, CacheStatus::Failed);
    assert!(harness.cache.contains("failed"));
    harness.handle.clear().await.expect("clear delivery");
    std::fs::remove_dir_all(harness.root).ok();
}
