mod delivery_fixture;
mod hls_terminal_wait;

use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness_with_requests;
use ghostr_engine::adaptive::{DecisionOutcome, RecordedWarpCommand};
use ghostr_engine::DeliveryKind;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use ghostr_net::outbound_media_client::MediaHttpRequests;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

struct PanicClient(Arc<AtomicUsize>);

impl MediaHttpRequests for PanicClient {
    fn get(&self, _url: &str) -> anyhow::Result<reqwest::RequestBuilder> {
        self.0.fetch_add(1, Ordering::Relaxed);
        panic!("fixture HLS transport panic")
    }
}

#[tokio::test]
async fn panicking_hls_fetch_has_one_terminal_decision_and_no_retry() {
    let calls = Arc::new(AtomicUsize::new(0));
    let requests = MediaRequestExecutor::new(
        Arc::new(PanicClient(Arc::clone(&calls))),
        MediaRequestLimits::try_new(2, 1).unwrap(),
    );
    let harness =
        start_harness_with_requests("hls-panicking-fetch", DeliveryOptions::default(), requests);
    let mut item = sized_item("stream", "https://panic.example/index.m3u8", 32, 4_000);
    item.meta.delivery = DeliveryKind::Hls;
    harness.handle.update_focus(focus_now(vec![item], 0, 0));

    let terminal = hls_terminal_wait::wait_terminal(&harness.segmented, "stream").await;
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
    assert_eq!(
        terminal.phase,
        ghostr_delivery::segmented::SegmentedPhase::Failed
    );
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    let records: Vec<_> = harness
        .handle
        .decision_history()
        .records
        .into_iter()
        .filter(hls_action)
        .collect();
    assert_eq!(records.len(), 1);
    assert!(matches!(
        &records[0].eventual_outcome,
        DecisionOutcome::Failed { class, .. } if class == "warp_hls_task_panicked"
    ));
    assert_eq!(records[0].actual_resources, None);
    std::fs::remove_dir_all(&harness.root).ok();
}

fn hls_action(record: &ghostr_engine::adaptive::DecisionRecord) -> bool {
    matches!(
        record
            .warp_decision
            .as_ref()
            .and_then(|decision| decision.selected.as_ref())
            .map(|selected| &selected.command),
        Some(RecordedWarpCommand::FetchHlsBootstrap { .. })
    )
}
