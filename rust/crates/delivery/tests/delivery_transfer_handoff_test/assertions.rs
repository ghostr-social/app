use crate::delivery_fixture::evidence::DeliveryEvidence as _;
use ghostr_delivery::delivery_events::DeliveryHandle;
use ghostr_engine::adaptive::{
    DecisionOutcome, DecisionRecord, RecordedPreemptionAuthority, RecordedWarpCommand,
};

pub fn assert_decisions(handle: &DeliveryHandle, cutoff: u64) {
    let records = handle.decision_history().records;
    assert!(
        !records.iter().any(|record| abandoned(record, cutoff)),
        "useful transfer is not abandoned"
    );
    assert_eq!(
        records
            .iter()
            .filter(|record| record.sequence > cutoff && bound_transfer(record))
            .count(),
        1,
        "handoff has one terminal completion"
    );
}

fn bound_transfer(record: &DecisionRecord) -> bool {
    playback_transfer(record)
        && record.chosen_action_id.is_some()
        && record.executed_request.is_some()
}

fn abandoned(record: &DecisionRecord, cutoff: u64) -> bool {
    record.sequence > cutoff
        && matches!(
            &record.eventual_outcome,
            DecisionOutcome::Failed { class, .. } if class == "decision_token_abandoned"
        )
}

fn playback_transfer(record: &DecisionRecord) -> bool {
    let command = record
        .warp_decision
        .as_ref()
        .and_then(|item| item.selected.as_ref())
        .map(|item| &item.command);
    matches!(
        command,
        Some(RecordedWarpCommand::Transfer { transfer })
            if transfer.authority == RecordedPreemptionAuthority::PlaybackCritical
    )
}
