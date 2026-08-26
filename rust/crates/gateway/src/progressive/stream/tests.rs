use super::{body_for_span, read_with_armed_change, wait_for_store_change, PumpStep, StreamSource};
use crate::progressive::capabilities::ProgressiveCapabilities;
use crate::progressive::route::{ProgressiveState, ProgressiveTiming};
use axum::body::to_bytes;
use core::future;
use core::time::Duration;
use ghostr_delivery::debug::network::NetworkThrottle;
use ghostr_delivery::playback_demand::demand_channel;
use ghostr_delivery::progressive_posts::ServablePosts;
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::{ContentRevision, PartialRangeStore};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{Mutex, Notify};

#[tokio::test]
async fn store_change_is_armed_before_the_range_read() {
    let notify = Arc::new(Notify::new());
    let during_read = std::sync::Arc::clone(&notify);
    let ((), changed) = read_with_armed_change(&notify, async move {
        during_read.notify_waiters();
    })
    .await;

    tokio::time::timeout(Duration::from_millis(10), changed)
        .await
        .expect("change emitted during the read must remain observable");
}

#[tokio::test(start_paused = true)]
async fn idle_deadline_wins_over_an_unrelated_store_change() {
    let result = wait_for_store_change(
        tokio::time::Instant::now(),
        future::ready(()),
        future::pending(),
    )
    .await;

    assert!(matches!(result, PumpStep::TimedOut));
}

#[tokio::test]
async fn a_store_failure_fails_the_promised_response_body() {
    let root = std::env::temp_dir().join(format!(
        "ghostr-gateway-store-error-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("valid test fixture")
            .as_nanos()
    ));
    std::fs::create_dir_all(root.join("clip.transform.video")).expect("valid test fixture");
    let (demand, _) = demand_channel();
    let state = Arc::new(ProgressiveState {
        store: Arc::new(PartialRangeStore::with_capacity(
            root.clone(),
            Arc::new(Mutex::new(0)),
            StoreCapacity::system(u64::MAX),
        )),
        demand,
        cache: ServablePosts::new(),
        network: NetworkThrottle::new(),
        timing: ProgressiveTiming::default(),
        capabilities: ProgressiveCapabilities::production(),
        #[cfg(all(
            feature = "video-debug-web",
            debug_assertions,
            not(any(target_os = "android", target_os = "ios"))
        ))]
        debug_feed: {
            let (delivery, _) = ghostr_delivery::delivery_events::command_channel();
            ghostr_delivery::debug::feed::DebugFeed::new(delivery, Vec::new())
        },
    });
    let source = StreamSource::new("clip".to_owned(), None, ContentRevision::default());

    let result = to_bytes(body_for_span(state, source, 0..1), usize::MAX).await;

    assert!(result.is_err());
    std::fs::remove_dir_all(root).expect("valid test fixture");
}
