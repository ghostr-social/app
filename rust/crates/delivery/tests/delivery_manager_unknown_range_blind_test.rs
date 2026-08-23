//! A lengthless range-blind origin gets one capped completion attempt.

mod delivery_fixture;

use delivery_fixture::items::{focus_now, unsized_item};
use delivery_fixture::media::{hit_log, hits};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::probe_origins::{serve_recording_range_blind, RANGE_BLIND_BODY};
use delivery_fixture::start_harness;
use delivery_fixture::wait::wait_for_ranges;
use ghostr_engine::EngineParams;
use std::time::Duration;

#[tokio::test]
async fn ignored_unknown_length_response_recovers_with_one_capped_whole_get() {
    let log = hit_log();
    let origin = serve_recording_range_blind(log.clone()).await;
    let mut options = DeliveryOptions::default();
    options.params = EngineParams {
        chunk_bytes: 4,
        ..options.params
    };
    options.tuning.retry.transient_attempts = 3;
    let harness = start_harness("ghostr-unknown-range-blind", options);

    harness
        .handle
        .update_focus(focus_now(vec![unsized_item("aa11", &origin)], 0, 0));

    wait_for_ranges(
        &harness.store,
        "aa11",
        &[(0, RANGE_BLIND_BODY.len() as u64)],
    )
    .await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    let requests = hits(&log);
    assert_eq!(
        requests
            .iter()
            .filter(|hit| hit.starts_with("GET:"))
            .count(),
        2,
        "range-blind body request repeated: {requests:?}"
    );
    assert!(requests.iter().any(|hit| hit == "GET:full"));
    assert_eq!(
        harness
            .store
            .read_range("aa11", 0..RANGE_BLIND_BODY.len() as u64)
            .await
            .unwrap()
            .unwrap(),
        RANGE_BLIND_BODY.to_vec()
    );
    std::fs::remove_dir_all(&harness.root).ok();
}
