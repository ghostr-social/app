//! Unsupported HLS features fail as local policy, not origin corruption.

mod delivery_fixture;
mod raw_http;

use core::time::Duration;
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_delivery::delivery_events::DeliveryHandle;
use ghostr_delivery::segmented::SegmentedPhase;
use ghostr_engine::DeliveryKind;

const WAIT_LIMIT: Duration = Duration::from_secs(10);
const ENCRYPTED_MANIFEST: &[u8] = b"HTTP/1.1 200 OK\r\n\
Content-Type: application/vnd.apple.mpegurl\r\n\
Content-Length: 81\r\n\
Connection: close\r\n\r\n\
#EXTM3U\n#EXT-X-KEY:METHOD=AES-128,URI=\"key\"\n\
#EXTINF:4,\nsegment.ts\n#EXT-X-ENDLIST\n";

#[tokio::test]
async fn encrypted_manifest_is_a_typed_media_policy_failure() {
    let responses = vec![ENCRYPTED_MANIFEST, ENCRYPTED_MANIFEST];
    let (source, requests) = raw_http::spawn_response_sequence(responses).await;
    let mut options = DeliveryOptions::default();
    options.tuning.retry.permanent_attempts = 2;
    let harness = start_harness("hls-unsupported-manifest", options);
    let mut item = sized_item("stream", &source, 81, 4_000);
    item.meta.delivery = DeliveryKind::Hls;
    harness.handle.update_focus(focus_now(vec![item], 0, 0));

    tokio::time::timeout(WAIT_LIMIT, requests)
        .await
        .expect("unsupported manifest retry")
        .expect("valid test fixture");
    let snapshot = wait_for_failure(&harness).await;
    assert_eq!(
        snapshot.detail.as_deref(),
        Some("HLS bootstrap was blocked by media policy")
    );
    assert_no_origin_feedback(&harness.handle);
    std::fs::remove_dir_all(&harness.root).ok();
}

fn assert_no_origin_feedback(handle: &DeliveryHandle) {
    let raw = handle.decision_history_json().expect("decision evidence");
    let json: serde_json::Value = serde_json::from_str(&raw).expect("decision schema");
    let records = json["decisions"]["records"]
        .as_array()
        .expect("decision records");
    let origins: Vec<_> = records
        .iter()
        .filter_map(|record| {
            record["warp_decision"]["planner_replay_capsule"]["origins"]["global"].as_array()
        })
        .collect();
    assert_eq!(origins.len(), records.len(), "missing ranking evidence");
    assert!(
        origins.iter().all(|records| records.is_empty()),
        "local HLS policy trained origin ranking: {origins:?}"
    );
}

async fn wait_for_failure(
    harness: &delivery_fixture::DeliveryHarness,
) -> ghostr_delivery::segmented::SegmentedSnapshot {
    let changed = harness.segmented.notifier();
    tokio::time::timeout(WAIT_LIMIT, async {
        loop {
            let changed = changed.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            let snapshot = harness.segmented.snapshot("stream");
            if snapshot.phase == SegmentedPhase::Failed {
                return snapshot;
            }
            changed.await;
        }
    })
    .await
    .expect("valid test fixture")
}
