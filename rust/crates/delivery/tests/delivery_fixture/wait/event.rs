use core::{ops::Range, time::Duration};
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use tokio::time::{timeout_at, Instant};

const WAIT_LIMIT: Duration = Duration::from_secs(10);

pub(super) async fn wait_for_ranges(store: &PartialRangeStore, key: &str, want: &[(u64, u64)]) {
    wait_until(store, key, |ranges| {
        want.iter()
            .all(|(start, end)| covered(ranges, *start, *end))
    })
    .await;
}

pub(super) async fn wait_until(
    store: &PartialRangeStore,
    key: &str,
    ready: impl Fn(&[Range<u64>]) -> bool,
) {
    let deadline = Instant::now() + WAIT_LIMIT;
    let notifier = store.change_notifier();
    loop {
        let changed = notifier.notified();
        tokio::pin!(changed);
        changed.as_mut().enable();
        let ranges = store.present_ranges(key).await.expect("present ranges");
        if ready(&ranges) {
            return;
        }
        if timeout_at(deadline, changed).await.is_err() {
            let latest = store.present_ranges(key).await.expect("present ranges");
            if ready(&latest) {
                return;
            }
            panic!("timed out waiting on ranges of {key}: {latest:?}");
        }
    }
}

pub(super) async fn wait_total_len(store: &PartialRangeStore, key: &str, expected: u64) {
    let deadline = Instant::now() + WAIT_LIMIT;
    let notifier = store.change_notifier();
    loop {
        let changed = notifier.notified();
        tokio::pin!(changed);
        changed.as_mut().enable();
        if store.total_len(key).await.expect("total len") == Some(expected) {
            return;
        }
        if timeout_at(deadline, changed).await.is_err() {
            let actual = store.total_len(key).await.expect("total len");
            if actual == Some(expected) {
                return;
            }
            panic!("timed out waiting on total length of {key}: {actual:?}");
        }
    }
}

pub(super) fn covered(ranges: &[Range<u64>], start: u64, end: u64) -> bool {
    ranges
        .iter()
        .any(|range| range.start <= start && range.end >= end)
}
