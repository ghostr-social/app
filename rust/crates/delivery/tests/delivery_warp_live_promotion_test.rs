mod delivery_fixture;
mod raw_http;

use delivery_fixture::decision::wait_for_history;
use delivery_fixture::evidence::DeliveryEvidence as _;
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use delivery_fixture::wait::wait_for_file;
use ghostr_engine::adaptive::{DecisionOutcome, RecordedWarpCommand};
use ghostr_engine::EngineParams;
use raw_http::spawn_gated_response;

const PROBE: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Type: video/mp4\r\nContent-Length: 16\r\nAccept-Ranges: bytes\r\nConnection: close\r\n\r\n";
const WHOLE: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Type: video/mp4\r\nContent-Length: 16\r\nConnection: close\r\n\r\n0123456789abcdef";

#[tokio::test]
async fn selected_promotion_authorizes_the_exact_live_request_before_headers() {
    let origin = spawn_gated_response(PROBE, WHOLE).await;
    let mut options = DeliveryOptions::default();
    options.params = EngineParams {
        chunk_bytes: 4,
        ..options.params
    };
    let harness = start_harness("warp-live-promotion", options);
    harness.handle.update_focus(focus_now(
        vec![sized_item("post", &origin.url, 16, 1_000)],
        0,
        5_000,
    ));

    let request = origin.body_request.await.expect("initial range request");
    assert!(String::from_utf8_lossy(&request).contains("range: bytes=0-3"));
    wait_for_history(&harness.handle, promotion_succeeded).await;
    let record = harness
        .handle
        .decision_history()
        .records
        .into_iter()
        .find(promotion_record)
        .expect("resolved promotion decision");
    assert_eq!(record.schema_version, 3);
    let selected = record
        .warp_decision
        .expect("valid test fixture")
        .selected
        .expect("valid test fixture");
    assert_eq!(selected.resources.network_bytes, 12);
    assert_eq!(selected.resources.storage_bytes, 12);
    assert_eq!(selected.resources.requests, 0);
    assert_exact_promotion(selected.command);

    origin.release_headers.notify_one();
    origin.requests.await.expect("probe and promoted request");
    wait_for_file(&harness.root.join("post.video")).await;
    assert_eq!(
        std::fs::read(harness.root.join("post.video")).expect("valid test fixture"),
        b"0123456789abcdef"
    );
    std::fs::remove_dir_all(&harness.root).expect("valid test fixture");
}

fn promotion_succeeded(
    history: &ghostr_delivery::delivery_events::DecisionHistorySnapshot,
) -> bool {
    history.records.iter().any(promotion_record)
}

fn promotion_record(record: &ghostr_engine::adaptive::DecisionRecord) -> bool {
    matches!(
        record.eventual_outcome,
        DecisionOutcome::Succeeded { bytes: 0, .. }
    ) && matches!(
        record
            .warp_decision
            .as_ref()
            .and_then(|item| item.selected.as_ref())
            .map(|item| &item.command),
        Some(RecordedWarpCommand::Promote { .. })
    )
}

fn assert_exact_promotion(command: RecordedWarpCommand) {
    let RecordedWarpCommand::Promote {
        action_id,
        source_id,
        grant,
        ..
    } = command
    else {
        panic!("expected promotion command");
    };
    assert!(action_id > 0, "promotion must target a live action");
    assert!(
        !source_id.is_empty(),
        "promotion must preserve its source identity"
    );
    assert_eq!(
        grant.maximum_bytes, 16,
        "promotion must authorize the unrequested remainder"
    );
    assert!(
        grant.valid_until_ms > 0,
        "promotion must carry a finite authorization deadline"
    );
}
