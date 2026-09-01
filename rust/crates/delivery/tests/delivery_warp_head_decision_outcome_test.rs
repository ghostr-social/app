//! A real selected HEAD resolves its original schema-v2 decision.

mod delivery_fixture;
mod raw_http;

use core::time::Duration;
use delivery_fixture::evidence::DeliveryEvidence as _;
use delivery_fixture::head_window::serve_visible_current;
use delivery_fixture::items::{focus_now, unsized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_delivery::delivery_events::{DecisionHistorySnapshot, DeliveryHandle};
use ghostr_engine::adaptive::{DecisionOutcome, RecordedWarpCommand};
use raw_http::spawn_raw_server;

const HEAD_RESPONSE: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 8\r\nAccept-Ranges: bytes\r\nContent-Type: video/mp4\r\nConnection: close\r\n\r\n";

#[tokio::test]
async fn learned_head_resolves_the_exact_authoritative_decision() {
    let (url, request) = spawn_raw_server(HEAD_RESPONSE).await;
    let harness = start_harness("warp-head-outcome", DeliveryOptions::default());
    let current = serve_visible_current().await;
    harness.handle.update_focus(focus_now(
        vec![current.item(), unsized_item("future", &url)],
        0,
        0,
    ));

    current.assert_get_without_head().await;
    let request = request.await.expect("valid test fixture");
    assert!(request.starts_with(b"HEAD "));
    let history = wait_for_head(&harness.handle).await;
    assert_head_record(&history);

    harness.handle.clear().await.expect("valid test fixture");
    std::fs::remove_dir_all(&harness.root).ok();
}

fn assert_head_record(history: &DecisionHistorySnapshot) {
    let record = history
        .records
        .iter()
        .find(|record| {
            matches!(
                record.eventual_outcome,
                DecisionOutcome::HeadObserved { .. }
            )
        })
        .expect("terminal HEAD decision");
    assert!(matches!(
        record.warp_decision.as_ref().and_then(|warp| warp.selected.as_ref()),
        Some(action) if matches!(action.command, RecordedWarpCommand::ProbeHead { .. })
    ));
    let DecisionOutcome::HeadObserved {
        content_length,
        accept_ranges,
        elapsed_ms,
    } = record.eventual_outcome
    else {
        panic!("terminal HEAD observation")
    };
    assert_eq!((content_length, accept_ranges), (8, Some(true)));
    assert!(elapsed_ms < 2_000);
}

async fn wait_for_head(handle: &DeliveryHandle) -> DecisionHistorySnapshot {
    let notifier = handle.plan_notifier();
    tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            let notified = notifier.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if handle.decision_history().records.iter().any(|record| {
                matches!(
                    record.eventual_outcome,
                    DecisionOutcome::HeadObserved { .. }
                )
            }) {
                return handle.decision_history();
            }
            notified.await;
        }
    })
    .await
    .expect("HEAD decision outcome")
}
