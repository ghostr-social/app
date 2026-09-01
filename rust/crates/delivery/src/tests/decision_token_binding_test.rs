use crate::delivery_events::command_channel;
use crate::tests::decision_log_fixture::{outcome, selected, work};
use ghostr_engine::adaptive::DecisionOutcome;
use ghostr_engine::ActionId;

#[test]
fn active_action_id_cannot_be_rebound_to_a_newer_decision() {
    let work = work();
    let (handle, commands) = command_channel();
    let (first_sequence, first) = selected(&handle, &commands, &work);
    assert!(commands.bind_decision(&first, ActionId::new(7), 100));
    let (second_sequence, second) = selected(&handle, &commands, &work);

    assert!(!commands.bind_decision(&second, ActionId::new(7), 200));
    commands
        .resolve_decision(
            ActionId::new(7),
            DecisionOutcome::Succeeded {
                bytes: 4,
                elapsed_ms: 0,
            },
            175,
        )
        .expect("first decision resolution");

    let history = handle.decision_history();
    assert_eq!(
        outcome(&history, first_sequence),
        &DecisionOutcome::Succeeded {
            bytes: 4,
            elapsed_ms: 75,
        }
    );
    assert_eq!(
        outcome(&history, second_sequence),
        &DecisionOutcome::Pending
    );
    assert!(commands.bind_decision(&second, ActionId::new(8), 200));
}

#[test]
fn superseded_token_cannot_bind_the_newest_decision() {
    let work = work();
    let (handle, commands) = command_channel();
    let (expired_sequence, expired) = selected(&handle, &commands, &work);
    let (_current_sequence, current) = selected(&handle, &commands, &work);

    assert!(!commands.bind_decision(&expired, ActionId::new(1), 100));
    assert!(commands.bind_decision(&current, ActionId::new(2), 100));
    let history = handle.decision_history();
    assert_eq!(
        outcome(&history, expired_sequence),
        &DecisionOutcome::Superseded
    );
    assert_eq!(
        history
            .records
            .last()
            .expect("valid test fixture")
            .chosen_action_id,
        Some(2)
    );
}

#[test]
fn token_from_another_log_cannot_bind_the_same_sequence() {
    let work = work();
    let (_first_handle, first) = command_channel();
    let (second_handle, second) = command_channel();
    let (_foreign_sequence, foreign) = selected(&_first_handle, &first, &work);
    let (_local_sequence, local) = selected(&second_handle, &second, &work);

    assert!(!second.bind_decision(&foreign, ActionId::new(1), 100));
    assert!(second.bind_decision(&local, ActionId::new(2), 100));
    assert_eq!(
        second_handle.decision_history().records[0].chosen_action_id,
        Some(2)
    );
}

#[test]
fn pending_is_not_a_terminal_resolution() {
    let work = work();
    let (handle, commands) = command_channel();
    let (sequence, token) = selected(&handle, &commands, &work);
    assert!(!commands.resolve_decision_token(&token, DecisionOutcome::Pending));
    assert!(commands.bind_decision(&token, ActionId::new(3), 100));
    assert!(commands
        .resolve_decision(ActionId::new(3), DecisionOutcome::Pending, 125)
        .is_none());
    assert_eq!(
        outcome(&handle.decision_history(), sequence),
        &DecisionOutcome::Pending
    );
    assert!(commands
        .resolve_decision(
            ActionId::new(3),
            DecisionOutcome::Succeeded {
                bytes: 5,
                elapsed_ms: 0,
            },
            150,
        )
        .is_some());
}
