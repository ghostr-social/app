use super::Scenario;
use crate::delivery_fixture::evidence::DeliveryEvidence as _;
use crate::delivery_fixture::full_recovery_origin::{PROBE_BYTES, TRIAL_BYTES};
use ghostr_engine::adaptive::{
    DecisionRecord, RecordedRetrievalRequest, RecordedWarpCommand, RecordedWholeBodyContract,
};

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
        assert!(
            history.records.iter().any(bound_capped_full),
            "full fallback owns its lease"
        );
        assert!(
            history.records.iter().any(bound_range),
            "bounded range owns its lease"
        );
    }

    pub(super) fn assert_trial_lease(&self) {
        let history = self.harness.handle.decision_history();
        assert_eq!(
            history
                .records
                .iter()
                .filter(|record| is_full(record))
                .count(),
            2,
            "one probe and one trial Full may be selected"
        );
        assert_eq!(
            history
                .records
                .iter()
                .filter(|record| executed_trial(record))
                .count(),
            1,
            "one Full recovery trial may execute"
        );
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

fn executed_trial(record: &DecisionRecord) -> bool {
    match record.executed_request.as_ref().map(|item| item.request) {
        Some(RecordedRetrievalRequest::FetchWhole {
            contract: RecordedWholeBodyContract::Exact { expected_bytes },
            ..
        }) => expected_bytes == TRIAL_BYTES as u64,
        Some(RecordedRetrievalRequest::FetchWhole {
            contract: RecordedWholeBodyContract::Capped { maximum_bytes },
            ..
        }) => maximum_bytes == TRIAL_BYTES as u64,
        _ => false,
    }
}

fn selected_request(record: &DecisionRecord) -> Option<&RecordedRetrievalRequest> {
    let selected = record.warp_decision.as_ref()?.selected.as_ref()?;
    match &selected.command {
        RecordedWarpCommand::Transfer { transfer } => Some(&transfer.request),
        _ => None,
    }
}
