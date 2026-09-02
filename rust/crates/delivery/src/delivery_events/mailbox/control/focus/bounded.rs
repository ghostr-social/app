use super::{
    feedback, focus_at, fold_passive_roster, folds_same_current, inherit_lineage, replace,
    same_current,
};
use crate::delivery_events::{DeliveryCommand, DeliveryFocus};
use std::collections::VecDeque;

pub(super) fn replace_three(
    pending: &mut VecDeque<DeliveryCommand>,
    indices: [usize; 3],
    mut update: DeliveryFocus,
) {
    let latest = focus_at(pending, indices[2]);
    inherit_lineage(&mut update, &latest);
    let carried = fold_passive_roster(&latest, &mut update);
    if carried || folds_same_current(&latest, &update) {
        replace_latest(pending, indices[2], update, carried);
        return;
    }
    make_room(pending, indices);
    replace(pending, update);
}

fn replace_latest(
    pending: &mut VecDeque<DeliveryCommand>,
    latest: usize,
    update: DeliveryFocus,
    carried: bool,
) {
    let removed = feedback::remove(pending, latest);
    if !carried {
        feedback::preserve(pending, &removed);
    }
    pending.push_back(DeliveryCommand::Focus(update));
}

fn make_room(pending: &mut VecDeque<DeliveryCommand>, indices: [usize; 3]) {
    let focuses = indices.map(|index| focus_at(pending, index));
    let removable = if same_current(&focuses[1], &focuses[2]) {
        indices[2]
    } else if same_current(&focuses[0], &focuses[1]) {
        indices[1]
    } else {
        indices[0]
    };
    feedback::remove_preserving(pending, removable);
}

pub(super) fn collapse_all(pending: &mut VecDeque<DeliveryCommand>, update: DeliveryFocus) {
    let indices = pending
        .iter()
        .enumerate()
        .filter_map(|(index, command)| {
            matches!(command, DeliveryCommand::Focus(_)).then_some(index)
        })
        .collect::<Vec<_>>();
    for index in indices.into_iter().rev() {
        feedback::remove_preserving(pending, index);
    }
    pending.push_back(DeliveryCommand::Focus(update));
}
