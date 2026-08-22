use ghostr_delivery::segmented::{SegmentedCache, SegmentedPhase, SegmentedSnapshot};
use std::time::Duration;

pub async fn wait_terminal(cache: &SegmentedCache, post: &str) -> SegmentedSnapshot {
    let changed = cache.notifier();
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let notified = changed.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            let snapshot = cache.snapshot(post);
            if matches!(
                snapshot.phase,
                SegmentedPhase::Ready | SegmentedPhase::Failed
            ) {
                return snapshot;
            }
            notified.await;
        }
    })
    .await
    .unwrap_or_else(|error| {
        panic!(
            "terminal HLS readiness: {error:?}; snapshot={:?}",
            cache.snapshot(post)
        )
    })
}
