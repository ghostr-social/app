use crate::delivery_events::command_channel;
use crate::tests::decision_log_fixture::{selected, work};
use ghostr_engine::adaptive::{DecisionOutcome, RecordedResourceCost, ResourceCost};
use ghostr_engine::ActionId;

#[test]
fn bound_terminal_resources_are_recorded_exactly_once() {
    let work = work();
    let (handle, commands) = command_channel();
    let (sequence, token) = selected(&handle, &commands, &work);
    let action = ActionId::new(7);
    assert!(commands.bind_decision(&token, action, 100));
    let actual = ResourceCost::new(0, 32, 7, 0);

    assert!(commands
        .resolve_decision_with_resources(
            action,
            DecisionOutcome::Succeeded {
                bytes: 32,
                elapsed_ms: 0,
            },
            actual,
            125,
        )
        .is_some());
    assert!(commands
        .resolve_decision_with_resources(action, DecisionOutcome::Superseded, actual, 150)
        .is_none());
    let record = handle
        .decision_history()
        .records
        .into_iter()
        .find(|record| record.sequence == sequence)
        .expect("valid test fixture");
    assert_eq!(
        record.actual_resources,
        Some(RecordedResourceCost::from(actual))
    );
}
