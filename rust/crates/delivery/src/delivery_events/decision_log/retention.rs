//! Completion-ordered retention for bounded, eventually observable decisions.

use super::HISTORY_CAPACITY;
use ghostr_engine::adaptive::{DecisionAction, DecisionOutcome, DecisionRecord};
use std::collections::VecDeque;

pub(super) fn trim(records: &mut VecDeque<DecisionRecord>, completed: &mut VecDeque<u64>) {
    while completed.len() > HISTORY_CAPACITY {
        let sequence = completed.pop_front().expect("completed decision");
        if let Some(index) = records
            .iter()
            .position(|record| record.sequence == sequence)
        {
            records.remove(index);
        }
    }
}

pub(super) fn supersede_unbound(records: &mut VecDeque<DecisionRecord>) -> Option<u64> {
    let record = records.iter_mut().rev().find(|record| {
        record.eventual_outcome == DecisionOutcome::Pending && record.chosen_action_id.is_none()
    })?;
    record
        .resolve(DecisionOutcome::Superseded)
        .then_some(record.sequence)
}

pub(super) fn resolve(
    records: &mut VecDeque<DecisionRecord>,
    sequence: u64,
    outcome: DecisionOutcome,
) -> Option<DecisionAction> {
    let record = records
        .iter_mut()
        .find(|record| record.sequence == sequence)?;
    let action = record.chosen_action.clone()?;
    record.resolve(outcome).then_some(action)
}

pub(super) fn resolve_unbound(
    records: &mut VecDeque<DecisionRecord>,
    sequence: u64,
    outcome: DecisionOutcome,
) -> Option<u64> {
    let record = records.iter_mut().find(|record| {
        record.sequence == sequence
            && record.eventual_outcome == DecisionOutcome::Pending
            && record.chosen_action_id.is_none()
    })?;
    record.resolve(outcome).then_some(sequence)
}
