//! Event-driven waits register before re-checking and use hard deadlines.

use core::{ops::Range, time::Duration};
use ghostr_delivery::cache_registry::CacheRegistry;
use ghostr_delivery::progressive_posts::ServablePosts;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use tokio::time::Instant;

mod event;

const WAIT_LIMIT: Duration = Duration::from_secs(10);

pub async fn wait_for_ranges(store: &PartialRangeStore, key: &str, want: &[(u64, u64)]) {
    event::wait_for_ranges(store, key, want).await;
}

pub async fn wait_until(
    store: &PartialRangeStore,
    key: &str,
    ready: impl Fn(&[Range<u64>]) -> bool,
) {
    event::wait_until(store, key, ready).await;
}

pub async fn wait_total_len(store: &PartialRangeStore, key: &str, expected: u64) {
    event::wait_total_len(store, key, expected).await;
}

pub fn covered(ranges: &[Range<u64>], start: u64, end: u64) -> bool {
    event::covered(ranges, start, end)
}

pub async fn wait_for_file(path: &std::path::Path) {
    wait_for_file_with_limit(path, WAIT_LIMIT).await;
}

pub async fn wait_for_file_with_limit(path: &std::path::Path, limit: Duration) {
    let deadline = Instant::now() + limit;
    while !path.exists() {
        assert!(
            Instant::now() < deadline,
            "timed out waiting for {}",
            path.display()
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

pub async fn wait_not_servable(posts: &ServablePosts, key: &str) {
    let deadline = Instant::now() + WAIT_LIMIT;
    while posts.contains(key) {
        assert!(
            Instant::now() < deadline,
            "timed out waiting for {key} to become unservable"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

pub async fn wait_cache_first(cache: &CacheRegistry, expected: &str) {
    let deadline = Instant::now() + WAIT_LIMIT;
    loop {
        if cache
            .videos()
            .first()
            .is_some_and(|video| video.id == expected)
        {
            return;
        }
        assert!(Instant::now() < deadline, "cache order did not update");
        tokio::task::yield_now().await;
    }
}
