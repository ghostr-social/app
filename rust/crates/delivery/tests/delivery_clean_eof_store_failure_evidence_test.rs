//! Clean network EOF remains exact evidence when local publication fails.

mod delivery_fixture;

use delivery_fixture::clean_eof_origin::{serve, BODY};
use delivery_fixture::decision::wait_for_history;
use delivery_fixture::items::{focus_now, unsized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use delivery_fixture::stats::wait_for;
use ghostr_engine::adaptive::{
    DecisionOutcome, DecisionRecord, RecordedRetrievalRequest, RecordedWarpCommand,
};
use ghostr_engine::host_stats::host_of;
use std::time::Duration;

const DIGEST: &str = "9f9f5111f7b27a781f1f1ddde5ebc2dd2b796bfc7365c9c28b548e564176929f";

#[tokio::test]
async fn clean_eof_teaches_size_without_claiming_unpublished_bytes_were_verified() {
    let mut origin = serve().await;
    let mut options = DeliveryOptions::default();
    options.tuning.retry.transient_attempts = 3;
    options.tuning.retry.base = Duration::from_millis(5);
    options.tuning.retry.max = Duration::from_millis(5);
    options.tuning.store_pressure_pause = Duration::from_millis(250);
    let harness = start_harness("ghostr-clean-eof-store-failure", options);
    let mut item = unsized_item("aa11", origin.url());
    item.meta.sha256 = Some(DIGEST.into());

    harness.handle.update_focus(focus_now(vec![item], 0, 0));
    origin.wait_whole_started().await;
    std::fs::create_dir(harness.root.join("aa11.response.ranges")).unwrap();
    origin.release();
    wait_for_history(&harness.handle, learned_after_failed_whole).await;
    let host = host_of(origin.url()).unwrap();
    let stats = wait_for(&harness.root.join("host_stats.json"), |stats| {
        stats.host_throughput(&host).is_some()
    })
    .await;
    tokio::time::sleep(Duration::from_millis(30)).await;

    assert_eq!(
        origin.gets(),
        2,
        "local failure must not redownload immediately"
    );
    assert_eq!(stats.failure_ratio(&host), 0.0);
    assert!(!harness.root.join("aa11.video").exists());
    assert!(!harness.root.join("aa11.verified").exists());
    assert_eq!(BODY.len(), 16);
    std::fs::remove_dir_all(&harness.root).ok();
}

fn learned_after_failed_whole(
    history: &ghostr_delivery::delivery_events::DecisionHistorySnapshot,
) -> bool {
    let Some(failed) = history.records.iter().position(failed_whole) else {
        return false;
    };
    history
        .records
        .iter()
        .skip(failed + 1)
        .any(has_exact_unverified_evidence)
}

fn failed_whole(record: &DecisionRecord) -> bool {
    matches!(record.eventual_outcome, DecisionOutcome::Failed { .. })
        && matches!(
            record
                .warp_decision
                .as_ref()
                .and_then(|warp| warp.selected.as_ref())
                .map(|action| &action.command),
            Some(RecordedWarpCommand::Transfer { transfer })
                if matches!(transfer.request, RecordedRetrievalRequest::FetchWhole { .. })
        )
}

fn has_exact_unverified_evidence(record: &DecisionRecord) -> bool {
    let value = serde_json::to_value(record).unwrap();
    let evidence = &value["replay_state"]["candidates"][0]["evidence"];
    evidence["size"]["exact"] == 16
        && !evidence["fields"]["AdvertisedHash"].is_null()
        && evidence["fields"]["Integrity"].is_null()
}
