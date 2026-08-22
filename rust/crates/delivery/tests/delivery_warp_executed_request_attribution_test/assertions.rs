use super::SLICE;
use ghostr_engine::adaptive::{
    DecisionRecord, RecordedPreemptionAuthority, RecordedResourceCost, RecordedRetrievalRequest,
    RecordedWarpCommand,
};

pub(super) fn assert_selected_slice(record: &DecisionRecord) {
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
            bytes_end: SLICE,
            promotion: None,
        }
    ),);
    assert_eq!(selected.resources, resources(SLICE));
}

pub(super) fn assert_executed_slice(record: &DecisionRecord) {
    let executed = record.executed_request.as_ref().unwrap();
    assert!(matches!(
        executed.request,
        RecordedRetrievalRequest::FetchRange {
            bytes_start: 0,
            bytes_end: SLICE,
            promotion: None,
        }
    ));
    assert_eq!(executed.resources, resources(SLICE));
}

fn resources(bytes: u64) -> RecordedResourceCost {
    RecordedResourceCost {
        network_bytes: bytes,
        storage_bytes: bytes,
        cpu_ms: 0,
        requests: 1,
    }
}
