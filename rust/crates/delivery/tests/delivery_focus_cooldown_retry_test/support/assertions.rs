use crate::delivery_fixture::evidence::DeliveryEvidence as _;
use ghostr_delivery::delivery_events::{DeliveryHandle, PlanEvidence};
use ghostr_engine::adaptive::DecisionOutcome;

pub fn assert_no_selected_action(handle: &DeliveryHandle, plan: &PlanEvidence) {
    let sequence = plan.decision_sequence.expect("plan decision sequence");
    let history = handle.decision_history();
    let record = history
        .records
        .iter()
        .find(|record| record.sequence == sequence)
        .expect("plan-linked decision");
    assert!(
        record
            .warp_decision
            .as_ref()
            .is_some_and(|decision| decision.selected.is_none()),
        "cooling or retired source selected executable work"
    );
    assert_eq!(
        record.eventual_outcome,
        DecisionOutcome::Succeeded {
            bytes: 0,
            elapsed_ms: 0,
        },
        "cooldown produces the expected terminal outcome"
    );
}
