mod delivery_fixture;

use core::time::Duration;
use delivery_fixture::evidence::DeliveryEvidence as _;
use delivery_fixture::hedge_tail_origin::HedgeTailOrigins;
use delivery_fixture::items::focus_now;
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::{start_harness_at, temp_directory};
use ghostr_delivery::delivery_events::FocusItem;
use ghostr_engine::adaptive::RecordedWarpCommand;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

const RANGE_BYTES: u64 = 64_000;
const TOTAL_BYTES: u64 = 1_000_000;

#[tokio::test]
async fn advertised_hash_cannot_hedge_a_sparse_stalled_primary() {
    let (origins, harness) = start_case().await;
    tokio::time::timeout(Duration::from_secs(1), origins.wait_primary())
        .await
        .expect("primary GET");
    assert!(
        tokio::time::timeout(Duration::from_millis(300), origins.wait_alternate())
            .await
            .is_err()
    );
    assert!(harness
        .handle
        .decision_history()
        .records
        .iter()
        .filter_map(|record| record.warp_decision.as_ref()?.selected.as_ref())
        .all(|action| !matches!(action.command, RecordedWarpCommand::Hedge { .. })));
    harness.handle.clear().await.expect("valid test fixture");
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
