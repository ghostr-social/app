use super::{ADMITTED, TOTAL};
use ghostr_engine::adaptive::{
    DecisionRecord, RecordedPreemptionAuthority, RecordedResourceCost, RecordedRetrievalRequest,
    RecordedWarpCommand,
};

pub(super) fn assert_selected_intent(record: &DecisionRecord) {
    let selected = record
        .warp_decision
        .as_ref()
        .unwrap()
        .selected
        .as_ref()
        .unwrap();
    let RecordedWarpCommand::Transfer { transfer } = &selected.command else {
        panic!("expected selected transfer");
    };
    assert_eq!(transfer.authority, RecordedPreemptionAuthority::Speculative);
    assert!(matches!(
        transfer.request,
        RecordedRetrievalRequest::FetchRange {
            bytes_start: 0,
            bytes_end: TOTAL,
            promotion: None,
        }
    ));
    assert_eq!(selected.resources, resources(TOTAL));
}

pub(super) fn assert_executed_cap(record: &DecisionRecord) {
    let executed = record.executed_request.as_ref().unwrap();
    assert!(matches!(
        executed.request,
        RecordedRetrievalRequest::FetchRange {
            bytes_start: 0,
            bytes_end: ADMITTED,
            promotion: None,
        }
    ));
    assert_eq!(executed.resources, resources(ADMITTED));
}

fn resources(bytes: u64) -> RecordedResourceCost {
    RecordedResourceCost {
        network_bytes: bytes,
        storage_bytes: bytes,
        cpu_ms: 0,
        requests: 1,
    }
}
