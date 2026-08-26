//! Event-driven waiting on the partial-range store: registers on the
//! change notifier before every re-check, with a hard deadline.

use core::ops::Range;
use core::time::Duration;
use ghostr_delivery::cache_registry::CacheRegistry;
use ghostr_delivery::progressive_posts::ServablePosts;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use tokio::time::{timeout_at, Instant};

const WAIT_LIMIT: Duration = Duration::from_secs(10);

pub async fn wait_for_ranges(store: &PartialRangeStore, key: &str, want: &[(u64, u64)]) {
    wait_until(store, key, |ranges| {
        want.iter()
            .all(|(start, end)| covered(ranges, *start, *end))
    })
    .await;
}

pub async fn wait_until(
    store: &PartialRangeStore,
    key: &str,
    ready: impl Fn(&[Range<u64>]) -> bool,
) {
    let deadline = Instant::now() + WAIT_LIMIT;
    let notify = store.change_notifier();
    loop {
        let changed = notify.notified();
        let ranges = store.present_ranges(key).await.expect("present ranges");
        if ready(&ranges) {
            return;
        }
        assert!(
            timeout_at(deadline, changed).await.is_ok(),
            "timed out waiting on ranges of {key}: {ranges:?}"
        );
    }
}

pub async fn wait_total_len(store: &PartialRangeStore, key: &str, expected: u64) {
    let deadline = Instant::now() + WAIT_LIMIT;
    let notify = store.change_notifier();
    loop {
        let changed = notify.notified();
        if store.total_len(key).await.expect("total len") == Some(expected) {
            return;
        }
        assert!(
            timeout_at(deadline, changed).await.is_ok(),
            "timed out waiting on the total length of {key}"
        );
    }
}

pub async fn wait_for_file(path: &std::path::Path) {
    let deadline = Instant::now() + WAIT_LIMIT;
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

pub fn covered(ranges: &[Range<u64>], start: u64, end: u64) -> bool {
    ranges
        .iter()
        .any(|range| range.start <= start && range.end >= end)
}
