//! An exhausted speculative whole-body cap advances geometrically, then stops at EOF.

mod delivery_fixture;
mod fast_start_mp4_fixture;

use delivery_fixture::items::{focus_now, unsized_item};
use delivery_fixture::media::{hit_log, hits};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::probe_origins::{serve_lengthless, serve_recording_range_blind_body};
use delivery_fixture::start_harness;
use delivery_fixture::wait::wait_for_ranges;
use ghostr_engine::adaptive::BOOTSTRAP_DIRECT_FETCH_BYTES;
use std::time::Duration;

#[tokio::test]
async fn exhausted_unknown_whole_fetch_grows_once_and_completes() {
    let log = hit_log();
    let body = padded_mp4(BOOTSTRAP_DIRECT_FETCH_BYTES as usize + 1);
    let origin = serve_recording_range_blind_body(log.clone(), body.clone()).await;
    let alternate = serve_lengthless().await;
    let mut options = DeliveryOptions::default();
    options.tuning.retry.transient_attempts = 4;
    let harness = start_harness("ghostr-unknown-whole-limit", options);
    let items = vec![
        unsized_item("aa11", &origin),
        unsized_item("bb22", &alternate),
    ];

    harness.handle.update_focus(focus_now(items.clone(), 0, 0));
    wait_for_ranges(&harness.store, "aa11", &[(0, body.len() as u64)]).await;
    harness.handle.update_focus(focus_now(items.clone(), 1, 0));
    harness.handle.update_focus(focus_now(items, 0, 0));
    tokio::time::sleep(Duration::from_millis(500)).await;

    let requests = hits(&log);
    assert_eq!(
        get_count(&requests),
        3,
        "unexpected probe growth: {requests:?}"
    );
    assert_eq!(requests.iter().filter(|hit| *hit == "GET:full").count(), 2);
    assert_eq!(
        harness
            .store
            .read_range("aa11", 0..body.len() as u64)
            .await
            .unwrap(),
        Some(body)
    );
    let efficiency = harness.handle.evaluation_snapshot().efficiency;
    assert_eq!(
        efficiency.aborted_bytes,
        BOOTSTRAP_DIRECT_FETCH_BYTES.saturating_add(1)
    );
    std::fs::remove_dir_all(&harness.root).ok();
}

fn get_count(requests: &[String]) -> usize {
    requests
        .iter()
        .filter(|hit| hit.starts_with("GET:"))
        .count()
}

fn padded_mp4(total: usize) -> Vec<u8> {
    let mut bytes = fast_start_mp4_fixture::front_indexed_mp4();
    let payload = total.saturating_sub(bytes.len()).saturating_sub(8);
    bytes.extend(((payload + 8) as u32).to_be_bytes());
    bytes.extend(b"free");
    bytes.resize(total, 0);
    bytes
}
