use super::delivery_fixture::decision::wait_for_history;
use super::delivery_fixture::evidence::DeliveryEvidence as _;
use super::delivery_fixture::DeliveryHarness;
use super::range_fixture::cancellation::CancellableOrigin;
use super::{PREFIX, TOTAL};
use core::sync::atomic::Ordering;
use core::time::Duration;
use ghostr_delivery::delivery_events::DeliveryHandle;
use ghostr_engine::adaptive::{
    DecisionOutcome, DecisionRecord, RecordedResourceCost, RecordedRetrievalRequest,
    RecordedWholeBodyContract, RecordedWholeFetchReason,
};
use ghostr_engine::RequestAuthority;

pub(super) fn pending_whole_sequence(handle: &DeliveryHandle) -> u64 {
    let history = handle.decision_history();
    let mut matches = history.records.iter().filter(|record| {
        record.eventual_outcome == DecisionOutcome::Pending
            && record.chosen_action_id.is_some()
            && record
                .executed_request
                .as_ref()
                .is_some_and(|executed| intended_whole(executed.request))
    });
    let sequence = matches.next().expect("bound whole decision").sequence;
    assert!(matches.next().is_none(), "one bound whole decision");
    sequence
}

pub(super) async fn assert_cancelled(harness: &DeliveryHarness, url: &str, sequence: u64) {
    wait_for_history(&harness.handle, |history| {
        history
            .records
            .iter()
            .any(|record| record.sequence == sequence && cancelled_prefix(record))
    })
    .await;
    let history = harness.handle.decision_history();
    let record = history
        .records
        .iter()
        .find(|record| record.sequence == sequence)
        .expect("retained cancelled whole decision");
    assert_eq!(record.actual_resources, Some(cancelled_resources()));
    let authority = RequestAuthority::from_url(url).expect("old request authority");
    assert_eq!(harness.requests.active_for(&authority), 0);
}

const fn cancelled_resources() -> RecordedResourceCost {
    RecordedResourceCost {
        network_bytes: PREFIX as u64,
        storage_bytes: 0,
        cpu_ms: 0,
        requests: 1,
    }
}

fn cancelled_prefix(record: &DecisionRecord) -> bool {
    matches!(
        record.eventual_outcome,
        DecisionOutcome::Cancelled { bytes, .. } if bytes == PREFIX as u64
    )
}

fn intended_whole(request: RecordedRetrievalRequest) -> bool {
    matches!(
        request,
        RecordedRetrievalRequest::FetchWhole {
            contract: RecordedWholeBodyContract::Capped {
                maximum_bytes: TOTAL
            },
            reason: RecordedWholeFetchReason::DirectCrossover,
        }
    )
}

pub(super) async fn assert_transport_stops(old: &CancellableOrigin, harness: &DeliveryHarness) {
    let before = harness
        .store
        .present_ranges("old")
        .await
        .expect("old ranges");
    assert!(before.is_empty(), "cancelled whole prefix must roll back");
    old.release.notify_one();
    tokio::time::timeout(Duration::from_secs(2), old.finished.notified())
        .await
        .expect("cancelled origin closes");
    let sent = old.bytes_sent.load(Ordering::SeqCst);
    assert!(sent < TOTAL, "cancelled origin drained {sent} bytes");
    let after = harness
        .store
        .present_ranges("old")
        .await
        .expect("old ranges");
    assert_eq!(after, before, "cancelled tail cannot become durable");
}
