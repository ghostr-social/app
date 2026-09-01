use core::ops::Range;
use core::time::Duration;
use ghostr_partial_store::partial_range_store::PartialRangeStore;

const WAIT_LIMIT: Duration = Duration::from_secs(30);
const DIAGNOSTIC_LIMIT: Duration = Duration::from_secs(5);
type StoredRanges = (Vec<Range<u64>>, Vec<Range<u64>>);

pub async fn wait_for_stored(store: &PartialRangeStore, post: &str, want: Range<u64>) {
    if tokio::time::timeout(WAIT_LIMIT, wait_stable(store, post, &want))
        .await
        .is_ok()
    {
        return;
    }
    let evidence = tokio::time::timeout(DIAGNOSTIC_LIMIT, stored_ranges(store, post)).await;
    if evidence
        .as_ref()
        .is_ok_and(|(_, stable)| covers(stable, &want))
    {
        return;
    }
    panic!("timed out storing {post} {want:?}; ranges={evidence:?}");
}

async fn wait_stable(store: &PartialRangeStore, post: &str, want: &Range<u64>) {
    let notifier = store.change_notifier();
    loop {
        let changed = notifier.notified();
        tokio::pin!(changed);
        changed.as_mut().enable();
        let (_, stable) = stored_ranges(store, post).await;
        if covers(&stable, want) {
            return;
        }
        changed.await;
    }
}

async fn stored_ranges(store: &PartialRangeStore, post: &str) -> StoredRanges {
    let snapshot = store.media_snapshot(post).await.expect("media snapshot");
    (
        snapshot.ranges().to_vec(),
        snapshot.planning_ranges().to_vec(),
    )
}

fn covers(ranges: &[Range<u64>], want: &Range<u64>) -> bool {
    ranges
        .iter()
        .any(|range| range.start <= want.start && range.end >= want.end)
}
