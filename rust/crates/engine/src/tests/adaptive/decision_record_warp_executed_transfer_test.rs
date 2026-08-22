use super::support::{bind, transfer_record};
use crate::adaptive::{DecisionReplayStatus, RecordedResourceCost, RecordedRetrievalRequest};

#[test]
fn selected_transfer_records_the_exact_smaller_executed_request() {
    let mut record = transfer_record();
    let selected_hash = record
        .replay_warp()
        .unwrap()
        .integrity()
        .decision_hash()
        .to_owned();

    assert!(bind(&mut record));

    let executed = record.executed_request.as_ref().unwrap();
    assert_eq!(record.chosen_action_id, Some(44));
    assert!(matches!(
        executed.request,
        RecordedRetrievalRequest::FetchRange {
            bytes_start: 16,
            bytes_end: 48,
            promotion: None
        }
    ));
    assert_eq!(
        executed.resources,
        RecordedResourceCost {
            network_bytes: 32,
            storage_bytes: 32,
            cpu_ms: 0,
            requests: 1,
        }
    );
    assert_eq!(record.replay(), DecisionReplayStatus::Verified);
    assert_eq!(
        record.replay_warp().unwrap().integrity().decision_hash(),
        selected_hash
    );
}
