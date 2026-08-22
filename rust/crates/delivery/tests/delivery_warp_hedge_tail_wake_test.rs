mod delivery_fixture;

use delivery_fixture::decision::wait_for_history;
use delivery_fixture::hedge_tail_assertions::{assert_exact_hedge, has_terminal_hedge};
use delivery_fixture::hedge_tail_origin::HedgeTailOrigins;
use delivery_fixture::items::focus_now;
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::{start_harness_at, temp_directory};
use ghostr_delivery::delivery_events::FocusItem;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::time::{Duration, Instant};

const RANGE_BYTES: u64 = 64_000;
const TOTAL_BYTES: u64 = 1_000_000;

#[tokio::test]
async fn exact_p95_wake_hedges_a_primary_stalled_before_headers() {
    let (origins, harness) = start_case().await;
    observe_tail_wake(&origins, &harness).await;
    wait_for_history(&harness.handle, |history| {
        has_terminal_hedge(history, RANGE_BYTES)
    })
    .await;
    assert_result(&harness);
    harness.handle.clear().await.unwrap();
    std::fs::remove_dir_all(&harness.root).ok();
}

async fn start_case() -> (HedgeTailOrigins, delivery_fixture::DeliveryHarness) {
    let origins = HedgeTailOrigins::serve(TOTAL_BYTES, RANGE_BYTES).await;
    let root = temp_directory("warp-hedge-tail-wake");
    delivery_fixture::hedge_tail_stats::seed(
        &root,
        &origins.primary_url,
        &origins.alternate_url,
        RANGE_BYTES,
    );
    let mut options = DeliveryOptions::default();
    options.params.chunk_bytes = RANGE_BYTES;
    let harness = start_harness_at(root, options);
    harness
        .handle
        .update_focus(focus_now(vec![mirrored_item(&origins)], 0, 5_000));
    (origins, harness)
}

async fn observe_tail_wake(
    origins: &HedgeTailOrigins,
    harness: &delivery_fixture::DeliveryHarness,
) {
    tokio::time::timeout(Duration::from_secs(1), origins.wait_primary())
        .await
        .expect("primary GET");
    let stalled_at = Instant::now();
    assert!(
        tokio::time::timeout(Duration::from_millis(60), origins.wait_alternate())
            .await
            .is_err()
    );
    let alternate =
        tokio::time::timeout(Duration::from_millis(500), origins.wait_alternate()).await;
    assert!(
        alternate.is_ok(),
        "p95 hedge GET; plans={:?}; decisions={:?}",
        harness.handle.plan_history(),
        harness.handle.decision_history()
    );
    assert!(stalled_at.elapsed() >= Duration::from_millis(100));
}

fn assert_result(harness: &delivery_fixture::DeliveryHarness) {
    assert_exact_hedge(&harness.handle.decision_history(), RANGE_BYTES);
    let efficiency = harness.handle.evaluation_snapshot().efficiency;
    assert!(efficiency.total_bytes >= RANGE_BYTES);
    assert_eq!(efficiency.duplicate_hedge_bytes, 0);
}

fn mirrored_item(origins: &HedgeTailOrigins) -> FocusItem {
    FocusItem {
        post: PostId::new("post"),
        meta: VideoMeta {
            urls: vec![origins.primary_url.clone(), origins.alternate_url.clone()],
            delivery: DeliveryKind::Progressive,
            sha256: Some("11".repeat(32)),
            size_bytes: Some(TOTAL_BYTES),
            duration_ms: Some(8_000),
        },
    }
}
