use crate::delivery_events::{command_channel, CommandReceiver};
use crate::manager::plan::PlannedWork;
use crate::tests::adaptive_plan_support::plan;
use ghostr_engine::adaptive::{AllocationPlan, DecisionOutcome, StorageSnapshot};
use ghostr_engine::ActionId;

#[test]
fn a_bound_decision_keeps_its_eventual_outcome_after_sixty_five_later_plans() {
    let work = work();
    let (handle, commands) = command_channel();
    let sequence = publish(&commands, &work, &work.plan);
    assert!(commands.bind_latest_decision(ActionId::new(7), 100));
    for _ in 0..65 {
        publish(&commands, &work, &AllocationPlan::default());
    }

    let outcome = DecisionOutcome::Succeeded {
        bytes: 8_192,
        elapsed_ms: 0,
    };
    let resolved = commands
        .resolve_decision(ActionId::new(7), outcome, 175)
        .expect("the bound decision must remain resolvable");
    let history = handle.decision_history();
    let record = history
        .records
        .iter()
        .find(|record| record.sequence == sequence)
        .expect("the completed outcome must remain observable");
    assert_eq!(history.records.len(), 64);
    assert_eq!(record.chosen_action.as_ref(), Some(&resolved.action));
    assert_eq!(resolved.elapsed_ms, 75);
    assert_eq!(
        record.eventual_outcome,
        DecisionOutcome::Succeeded {
            bytes: 8_192,
            elapsed_ms: 75,
        }
    );

    publish(&commands, &work, &AllocationPlan::default());
    assert!(handle
        .decision_history()
        .records
        .iter()
        .any(|record| record.sequence == sequence));
}

#[test]
fn pending_decisions_are_additional_to_the_terminal_history_bound() {
    let work = work();
    let (handle, commands) = command_channel();
    let mut first = 0;
    for action in 1..=65 {
        let sequence = publish(&commands, &work, &work.plan);
        first = if first == 0 { sequence } else { first };
        assert!(commands.bind_latest_decision(ActionId::new(action), 100));
    }
    let newest = publish(&commands, &work, &work.plan);

    let history = handle.decision_history();
    assert_eq!(history.records.len(), 66);
    assert!(history
        .records
        .iter()
        .any(|record| record.sequence == first));
    let pending = history.records.last().expect("latest decision");
    assert_eq!(pending.sequence, newest);
    assert_eq!(pending.eventual_outcome, DecisionOutcome::Pending);
    assert_eq!(pending.chosen_action_id, None);
}

fn work() -> PlannedWork {
    plan(20_000, 4_000_000, StorageSnapshot::new(2_000_000_000, 0))
}

fn publish(commands: &CommandReceiver, work: &PlannedWork, allocation: &AllocationPlan) -> u64 {
    commands.publish_decision(
        work.snapshot.as_ref().expect("planning snapshot"),
        allocation,
        work.shadow_prices,
        &work.decision_models,
    )
}
