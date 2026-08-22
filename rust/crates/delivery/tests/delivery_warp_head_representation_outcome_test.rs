//! A claimed HEAD keeps exact authority until its stale response terminates.

mod delivery_fixture;
mod raw_http;

use delivery_fixture::decision::wait_for_history;
use delivery_fixture::items::{focus_now, unsized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_delivery::delivery_events::DeliveryHandle;
use ghostr_engine::adaptive::{DecisionOutcome, RecordedWarpCommand};
use raw_http::{spawn_raw_server, spawn_stalled_headers};
use std::time::Duration;

const HEAD_RESPONSE: &[u8] =
    b"HTTP/1.1 200 OK\r\nContent-Length: 8\r\nAccept-Ranges: bytes\r\nConnection: close\r\n\r\n";

#[tokio::test]
async fn replaced_head_stays_pending_then_resolves_as_superseded() {
    let old = spawn_stalled_headers().await;
    let (new_url, new_request) = spawn_raw_server(HEAD_RESPONSE).await;
    let harness = start_harness("warp-head-representation", DeliveryOptions::default());
    harness
        .handle
        .update_focus(focus_now(vec![unsized_item("post", &old.url)], 0, 0));
    old.request_started.await.expect("old HEAD starts");
    let sequence = pending_head(&harness.handle);

    harness
        .handle
        .update_focus(focus_now(vec![unsized_item("post", &new_url)], 0, 0));
    wait_for_later_decision(&harness.handle, sequence).await;
    assert_eq!(outcome(&harness.handle, sequence), DecisionOutcome::Pending);

    old.requests.abort();
    let _ = old.requests.await;
    tokio::time::timeout(Duration::from_secs(2), new_request)
        .await
        .expect("replacement request starts")
        .expect("replacement request");
    wait_for_superseded(&harness.handle, sequence).await;

    harness.handle.clear().await.unwrap();
    std::fs::remove_dir_all(&harness.root).ok();
}

fn pending_head(handle: &DeliveryHandle) -> u64 {
    handle
        .decision_history()
        .records
        .iter()
        .rev()
        .find(|record| {
            record.eventual_outcome == DecisionOutcome::Pending
                && record.warp_decision.as_ref().is_some_and(|warp| {
                    warp.selected.as_ref().is_some_and(|action| {
                        matches!(action.command, RecordedWarpCommand::ProbeHead { .. })
                    })
                })
        })
        .expect("pending HEAD record")
        .sequence
}

async fn wait_for_later_decision(handle: &DeliveryHandle, sequence: u64) {
    wait_for_history(handle, |history| {
        history
            .records
            .last()
            .is_some_and(|record| record.sequence > sequence)
    })
    .await;
}

async fn wait_for_superseded(handle: &DeliveryHandle, sequence: u64) {
    wait_for_history(handle, |history| {
        history.records.iter().any(|record| {
            record.sequence == sequence && record.eventual_outcome == DecisionOutcome::Superseded
        })
    })
    .await;
}

fn outcome(handle: &DeliveryHandle, sequence: u64) -> DecisionOutcome {
    handle
        .decision_history()
        .records
        .into_iter()
        .find(|record| record.sequence == sequence)
        .expect("decision record")
        .eventual_outcome
}
