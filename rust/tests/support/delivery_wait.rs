//! Event-driven waiting on the partial-range store: registers on the
//! change notifier before every re-check, with a hard deadline.

use rust_lib_ghostr::video::partial_range_store::PartialRangeStore;
use std::ops::Range;
use std::time::Duration;
use tokio::time::{timeout_at, Instant};

const WAIT_LIMIT: Duration = Duration::from_secs(10);

pub async fn wait_for_ranges(store: &PartialRangeStore, key: &str, want: &[(u64, u64)]) {
    wait_until(store, key, |ranges| {
        want.iter().all(|(start, end)| covered(ranges, *start, *end))
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
        if timeout_at(deadline, changed).await.is_err() {
            panic!("timed out waiting on ranges of {key}: {ranges:?}");
        }
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
        if timeout_at(deadline, changed).await.is_err() {
            panic!("timed out waiting on the total length of {key}");
        }
    }
}

pub async fn wait_for_file(path: &std::path::Path) {
    let deadline = Instant::now() + WAIT_LIMIT;
    while !path.exists() {
        if Instant::now() >= deadline {
            panic!("timed out waiting for {}", path.display());
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

pub fn covered(ranges: &[Range<u64>], start: u64, end: u64) -> bool {
    ranges
        .iter()
        .any(|range| range.start <= start && range.end >= end)
}
