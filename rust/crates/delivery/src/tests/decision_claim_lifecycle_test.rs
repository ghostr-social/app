use crate::delivery_events::command_channel;
use crate::tests::decision_log_fixture::{head_identity, outcome, publish, selected_head, work};
use ghostr_engine::adaptive::{AllocationPlan, DecisionOutcome};

#[test]
fn claimed_head_survives_history_pressure_and_resolves_exactly() {
    let (handle, commands) = command_channel();
    let (sequence, token) = selected_head(&handle, &commands);
    let identity = head_identity();
    let claim = commands
        .claim_decision(token, &identity, 100)
        .unwrap_or_else(|_| panic!("HEAD decision claim"));
    let work = work();
    for _ in 0..65 {
        let (_, token) = publish(&handle, &commands, &work, &AllocationPlan::default());
        assert!(token.is_none());
    }
    assert_eq!(
        outcome(&handle.decision_history(), sequence),
        &DecisionOutcome::Pending
    );

    commands
        .resolve_decision_claim(
            claim,
            DecisionOutcome::HeadObserved {
                content_length: 8,
                accept_ranges: Some(true),
                elapsed_ms: 0,
            },
            175,
        )
        .expect("claimed HEAD resolution");
    assert_eq!(
        outcome(&handle.decision_history(), sequence),
        &DecisionOutcome::HeadObserved {
            content_length: 8,
            accept_ranges: Some(true),
            elapsed_ms: 75,
        }
    );
    assert_eq!(handle.decision_history().records.len(), 64);
}

#[test]
fn dropped_dispatch_and_claim_handles_terminally_fail_the_record() {
    let (handle, commands) = command_channel();
    let (dispatch_sequence, token) = selected_head(&handle, &commands);
    drop(token);
    assert_failed(&handle, dispatch_sequence, "decision_token_abandoned");

    let (claim_sequence, token) = selected_head(&handle, &commands);
    let started_at_ms = unix_time_ms().saturating_sub(75);
    let identity = head_identity();
    let claim = commands
        .claim_decision(token, &identity, started_at_ms)
        .unwrap_or_else(|_| panic!("HEAD decision claim"));
    drop(claim);
    assert_abandoned_claim(&handle, claim_sequence);
}

fn assert_abandoned_claim(handle: &crate::delivery_events::DeliveryHandle, sequence: u64) {
    let history = handle.decision_history();
    let DecisionOutcome::Failed { class, elapsed_ms } = outcome(&history, sequence) else {
        panic!("abandoned claim failure")
    };
    assert_eq!(class, "warp_head_probe_abandoned");
    assert!(*elapsed_ms >= 75);
}

fn unix_time_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn assert_failed(handle: &crate::delivery_events::DeliveryHandle, sequence: u64, expected: &str) {
    assert_eq!(
        outcome(&handle.decision_history(), sequence),
        &DecisionOutcome::Failed {
            class: expected.into(),
            elapsed_ms: 0,
        }
    );
}
