//! Completion-ordered retention for bounded, eventually observable decisions.

use super::HISTORY_CAPACITY;
use ghostr_engine::adaptive::{
    DecisionAction, DecisionOutcome, DecisionRecord, RecordedWarpAction,
};
use std::collections::{HashSet, VecDeque};

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

pub(super) fn supersede_unbound(
    records: &mut VecDeque<DecisionRecord>,
    claimed: &HashSet<u64>,
) -> Option<u64> {
    let record = records.iter_mut().rev().find(|record| {
        record.eventual_outcome == DecisionOutcome::Pending
            && record.chosen_action_id.is_none()
            && !claimed.contains(&record.sequence)
    })?;
    record
        .resolve(DecisionOutcome::Superseded)
        .then_some(record.sequence)
}

pub(super) fn resolve(
    records: &mut VecDeque<DecisionRecord>,
    sequence: u64,
    outcome: DecisionOutcome,
) -> Option<(DecisionAction, Option<RecordedWarpAction>)> {
    let record = records
        .iter_mut()
        .find(|record| record.sequence == sequence)?;
    let action = record.chosen_action.clone()?;
    let warp_action = record
        .warp_decision
        .as_ref()
        .and_then(|decision| decision.selected.clone());
    record.resolve(outcome).then_some((action, warp_action))
}

pub(super) fn resolve_unbound(
    records: &mut VecDeque<DecisionRecord>,
    claimed: bool,
    sequence: u64,
    outcome: DecisionOutcome,
) -> Option<u64> {
    let record = records.iter_mut().find(|record| {
        record.sequence == sequence
            && record.eventual_outcome == DecisionOutcome::Pending
            && record.chosen_action_id.is_none()
            && !claimed
    })?;
    record.resolve(outcome).then_some(sequence)
}

pub(super) fn resolve_claimed(
    records: &mut VecDeque<DecisionRecord>,
    claimed: bool,
    sequence: u64,
    outcome: DecisionOutcome,
) -> Option<(DecisionAction, Option<RecordedWarpAction>)> {
    claimed.then_some(())?;
    resolve(records, sequence, outcome)
}
