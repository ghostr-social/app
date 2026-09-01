//! A verified real-manager completion reaches replayable integrity evidence.

mod delivery_fixture;

use delivery_fixture::clean_eof_origin::serve;
use delivery_fixture::decision::history::wait_for_history_with_limit;
use delivery_fixture::items::{focus_now, unsized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use std::time::Duration;

const DIGEST: &str = "9f9f5111f7b27a781f1f1ddde5ebc2dd2b796bfc7365c9c28b548e564176929f";
const EVIDENCE_WAIT_LIMIT: Duration = Duration::from_secs(30);

#[tokio::test]
async fn clean_eof_verified_bytes_become_integrity_evidence() {
    let mut origin = serve().await;
    let harness = start_harness("ghostr-clean-eof-verified", DeliveryOptions::default());
    let mut item = unsized_item("aa11", origin.url());
    item.meta.sha256 = Some(DIGEST.into());

    harness.handle.update_focus(focus_now(vec![item], 0, 0));
    origin.wait_whole_started().await;
    origin.release();
    wait_for_history_with_limit(&harness.handle, EVIDENCE_WAIT_LIMIT, has_verified_complete).await;

    assert_eq!(origin.gets(), 2, "range recovery must precede whole GET");
    assert!(harness.root.join("aa11.video").exists());
    assert!(harness.root.join("aa11.verified").exists());
    std::fs::remove_dir_all(&harness.root).ok();
}

fn has_verified_complete(
    history: &ghostr_delivery::delivery_events::DecisionHistorySnapshot,
) -> bool {
    history.records.iter().any(|record| {
        let value = serde_json::to_value(record).expect("valid test fixture");
        let evidence = &value["replay_state"]["candidates"][0]["evidence"];
        evidence["size"]["exact"] == 16 && !evidence["fields"]["Integrity"].is_null()
    })
}
