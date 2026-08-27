use super::Scenario;
use crate::delivery_fixture::evidence::DeliveryEvidence as _;
use crate::delivery_fixture::full_recovery_origin::PROBE_BYTES;
use ghostr_engine::adaptive::{DecisionRecord, RecordedRetrievalRequest, RecordedWarpCommand};

impl Scenario {
    pub(super) async fn assert_method_specific_lease(&self) {
        tokio::time::sleep(core::time::Duration::from_millis(50)).await;
        let history = self.harness.handle.decision_history();
        assert_eq!(
            history
                .records
                .iter()
                .filter(|record| is_full(record))
                .count(),
            1,
            "only the leased Full may be selected"
        );
        assert!(history.records.iter().any(bound_capped_full));
        assert!(history.records.iter().any(bound_range));
    }
}

fn bound_capped_full(record: &DecisionRecord) -> bool {
    is_full(record)
        && matches!(
            record.executed_request.as_ref().map(|item| &item.request),
            Some(RecordedRetrievalRequest::FetchRange {
                bytes_start: 0,
                bytes_end,
                ..
            }) if *bytes_end == PROBE_BYTES as u64
        )
}

fn is_full(record: &DecisionRecord) -> bool {
    selected_request(record)
        .is_some_and(|request| matches!(request, RecordedRetrievalRequest::FetchWhole { .. }))
}

fn bound_range(record: &DecisionRecord) -> bool {
    record.executed_request.is_some()
        && selected_request(record)
            .is_some_and(|request| matches!(request, RecordedRetrievalRequest::FetchRange { .. }))
}

fn selected_request(record: &DecisionRecord) -> Option<&RecordedRetrievalRequest> {
    let selected = record.warp_decision.as_ref()?.selected.as_ref()?;
    match &selected.command {
        RecordedWarpCommand::Transfer { transfer } => Some(&transfer.request),
        _ => None,
    }
}
