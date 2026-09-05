use super::ADMITTED;
use ghostr_engine::adaptive::{
    DecisionRecord, RecordedPreemptionAuthority, RecordedResourceCost, RecordedRetrievalRequest,
    RecordedWarpCommand,
};

pub(super) fn assert_selected_intent(record: &DecisionRecord) {
    let selected = record
        .warp_decision
        .as_ref()
        .expect("valid test fixture")
        .selected
        .as_ref()
        .expect("valid test fixture");
    let RecordedWarpCommand::Transfer { transfer } = &selected.command else {
        panic!("expected selected transfer");
    };
    assert_eq!(
        transfer.authority,
        RecordedPreemptionAuthority::Transition,
        "the selected intent must retain the bounded encoded-window authority"
    );
    assert!(
        matches!(
            transfer.request,
            RecordedRetrievalRequest::FetchRange {
                bytes_start: 0,
                bytes_end: ADMITTED,
                promotion: None,
            }
        ),
        "selected: {selected:#?}; executed: {:#?}",
        record.executed_request
    );
    assert_eq!(
        selected.resources,
        resources(ADMITTED),
        "selected resources must describe the requested range"
    );
}

pub(super) fn assert_executed_cap(record: &DecisionRecord) {
    let executed = record
        .executed_request
        .as_ref()
        .expect("valid test fixture");
    assert!(
        matches!(
            executed.request,
            RecordedRetrievalRequest::FetchRange {
                bytes_start: 0,
                bytes_end: ADMITTED,
                promotion: None,
            }
        ),
        "the executed request must record the admitted cap"
    );
    assert_eq!(
        executed.resources,
        resources(ADMITTED),
        "executed resources must describe the admitted request"
    );
}

fn resources(bytes: u64) -> RecordedResourceCost {
    RecordedResourceCost {
        network_bytes: bytes,
        storage_bytes: bytes,
        cpu_ms: 0,
        requests: 1,
    }
}
