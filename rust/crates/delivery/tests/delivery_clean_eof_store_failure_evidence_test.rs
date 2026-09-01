//! Clean network EOF remains exact evidence when local publication fails.

mod delivery_fixture;
#[path = "delivery_clean_eof_store_failure_evidence_test/support.rs"]
mod support;

use delivery_fixture::clean_eof_origin::{serve, BODY};
use delivery_fixture::items::focus_now;
use delivery_fixture::start_harness;
use ghostr_engine::adaptive::{
    DecisionOutcome, DecisionRecord, RecordedRetrievalRequest, RecordedWarpCommand,
};
use support::{assert_failure_evidence, block_publication, hashed_item, options};

const DIGEST: &str = "9f9f5111f7b27a781f1f1ddde5ebc2dd2b796bfc7365c9c28b548e564176929f";

#[tokio::test]
async fn clean_eof_teaches_size_without_claiming_unpublished_bytes_were_verified() {
    let mut origin = serve().await;
    let harness = start_harness("ghostr-clean-eof-store-failure", options());

    harness
        .handle
        .update_focus(focus_now(vec![hashed_item(origin.url())], 0, 0));
    origin.wait_whole_started().await;
    block_publication(&harness);
    origin.release();
    assert_failure_evidence(&harness, origin.url()).await;

    assert_eq!(
        origin.gets(),
        2,
        "local failure must not redownload immediately"
    );
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
    let value = serde_json::to_value(record).expect("valid test fixture");
    let evidence = &value["replay_state"]["candidates"][0]["evidence"];
    evidence["size"]["exact"] == 16
        && !evidence["fields"]["AdvertisedHash"].is_null()
        && evidence["fields"]["Integrity"].is_null()
}
