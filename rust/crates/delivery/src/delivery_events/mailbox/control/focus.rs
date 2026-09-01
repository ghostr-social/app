use crate::delivery_events::{DeliveryCommand, DeliveryFocus, FocusTransition};
use core::cmp::Ordering;
use std::collections::VecDeque;

mod bounded;
mod feedback;

#[derive(Clone, Copy)]
enum PairAction {
    CarryLatest,
    ReplaceLatest,
    PromoteLatest,
    Collapse,
    Append,
}

pub(super) fn replace(pending: &mut VecDeque<DeliveryCommand>, update: DeliveryFocus) {
    let indices = focus_indices(pending);
    match indices.as_slice() {
        [] => pending.push_back(DeliveryCommand::Focus(update)),
        [index] => replace_one(pending, *index, update),
        [anchor, latest] => replace_pair(pending, *anchor, *latest, update),
        [anchor, middle, latest] => {
            bounded::replace_three(pending, [*anchor, *middle, *latest], update);
        }
        _ => bounded::collapse_all(pending, update),
    }
}

pub(super) fn compact_for_delivery(pending: &mut VecDeque<DeliveryCommand>) {
    let indices = focus_indices(pending);
    let [anchor, latest] = indices.as_slice() else {
        return;
    };
    let first = focus_at(pending, *anchor);
    let last = focus_at(pending, *latest);
    if !retains_ordered_edge(&first, &last) {
        feedback::remove_preserving(pending, *anchor);
    }
}

fn replace_one(pending: &mut VecDeque<DeliveryCommand>, index: usize, mut update: DeliveryFocus) {
    let previous = focus_at(pending, index);
    inherit_lineage(&mut update, &previous);
    let carried = fold_passive_roster(&previous, &mut update);
    if carried || folds_same_current(&previous, &update) {
        let removed = feedback::remove(pending, index);
        if !carried {
            feedback::preserve(pending, &removed);
        }
    }
    pending.push_back(DeliveryCommand::Focus(update));
}

fn replace_pair(
    pending: &mut VecDeque<DeliveryCommand>,
    anchor_index: usize,
    latest_index: usize,
    mut update: DeliveryFocus,
) {
    let anchor = focus_at(pending, anchor_index);
    let latest = focus_at(pending, latest_index);
    inherit_lineage(&mut update, &latest);
    let action = pair_action(&anchor, &latest, &mut update);
    apply_pair_action(pending, (anchor_index, latest_index), update, action);
}

fn pair_action(
    anchor: &DeliveryFocus,
    latest: &DeliveryFocus,
    update: &mut DeliveryFocus,
) -> PairAction {
    if fold_passive_roster(latest, update) {
        return PairAction::CarryLatest;
    }
    if same_current(latest, update) {
        return same_current_action(latest, update);
    }
    if latest.transition != update.transition {
        return PairAction::Append;
    }
    navigation_action(anchor, latest, update)
}

fn same_current_action(latest: &DeliveryFocus, update: &DeliveryFocus) -> PairAction {
    if latest.transition == update.transition || latest.transition == FocusTransition::RosterChange
    {
        PairAction::ReplaceLatest
    } else {
        PairAction::Append
    }
}

fn navigation_action(
    anchor: &DeliveryFocus,
    latest: &DeliveryFocus,
    update: &DeliveryFocus,
) -> PairAction {
    match (direction(anchor, latest), direction(latest, update)) {
        (Some(Ordering::Greater), Some(Ordering::Less)) => PairAction::PromoteLatest,
        (Some(Ordering::Less), _) => correction_action(anchor, update),
        _ => PairAction::ReplaceLatest,
    }
}

fn correction_action(anchor: &DeliveryFocus, update: &DeliveryFocus) -> PairAction {
    match direction(anchor, update) {
        Some(Ordering::Less) => PairAction::ReplaceLatest,
        Some(Ordering::Equal | Ordering::Greater) => PairAction::Collapse,
        None => PairAction::ReplaceLatest,
    }
}

fn apply_pair_action(
    pending: &mut VecDeque<DeliveryCommand>,
    indices: (usize, usize),
    update: DeliveryFocus,
    action: PairAction,
) {
    let (anchor, latest) = indices;
    match action {
        PairAction::CarryLatest => {
            feedback::remove(pending, latest);
        }
        PairAction::ReplaceLatest => feedback::remove_preserving(pending, latest),
        PairAction::PromoteLatest => feedback::remove_preserving(pending, anchor),
        PairAction::Collapse => feedback::remove_pair_preserving(pending, anchor, latest),
        PairAction::Append => {}
    }
    pending.push_back(DeliveryCommand::Focus(update));
}

pub(super) fn folds_same_current(previous: &DeliveryFocus, update: &DeliveryFocus) -> bool {
    same_current(previous, update)
        && (previous.transition == update.transition
            || previous.transition == FocusTransition::RosterChange)
}

pub(super) fn fold_passive_roster(previous: &DeliveryFocus, update: &mut DeliveryFocus) -> bool {
    if !same_current(previous, update) || update.transition != FocusTransition::RosterChange {
        return false;
    }
    update.transition = previous.transition;
    update.rescue = previous.rescue;
    true
}

pub(super) fn inherit_lineage(update: &mut DeliveryFocus, previous: &DeliveryFocus) {
    if same_current(previous, update) {
        update.generation = update.generation.covering(previous.generation);
    }
}

fn retains_ordered_edge(first: &DeliveryFocus, last: &DeliveryFocus) -> bool {
    first.transition != last.transition || direction(first, last) == Some(Ordering::Less)
}

fn direction(from: &DeliveryFocus, to: &DeliveryFocus) -> Option<Ordering> {
    let previous = from.current_post()?;
    let previous_index = to.items.iter().position(|item| &item.post == previous)?;
    let current_index = to.current_index.min(to.items.len().checked_sub(1)?);
    Some(current_index.cmp(&previous_index))
}

pub(super) fn same_current(left: &DeliveryFocus, right: &DeliveryFocus) -> bool {
    right.current_post().is_some() && right.current_post() == left.current_post()
}

fn focus_indices(pending: &VecDeque<DeliveryCommand>) -> Vec<usize> {
    pending
        .iter()
        .enumerate()
        .filter_map(|(index, command)| {
            matches!(command, DeliveryCommand::Focus(_)).then_some(index)
        })
        .collect()
}

pub(super) fn focus_at(pending: &VecDeque<DeliveryCommand>, index: usize) -> DeliveryFocus {
    let Some(DeliveryCommand::Focus(focus)) = pending.get(index) else {
        unreachable!("focus index")
    };
    focus.clone()
}
