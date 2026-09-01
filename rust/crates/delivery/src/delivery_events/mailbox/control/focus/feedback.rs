use crate::delivery_events::{
    DeliveryCommand, DeliveryFocus, FocusTransition, TransportRescueFeedback,
};
use std::collections::VecDeque;

pub(super) fn remove(pending: &mut VecDeque<DeliveryCommand>, index: usize) -> DeliveryFocus {
    let Some(DeliveryCommand::Focus(focus)) = pending.remove(index) else {
        unreachable!("focus index")
    };
    focus
}

pub(super) fn remove_preserving(pending: &mut VecDeque<DeliveryCommand>, index: usize) {
    let removed = remove(pending, index);
    preserve(pending, &removed);
}

pub(super) fn remove_pair_preserving(
    pending: &mut VecDeque<DeliveryCommand>,
    first: usize,
    last: usize,
) {
    let last = remove(pending, last);
    let first = remove(pending, first);
    preserve(pending, &first);
    preserve(pending, &last);
}

pub(super) fn preserve(pending: &mut VecDeque<DeliveryCommand>, focus: &DeliveryFocus) {
    let Some(rescue) = focus
        .rescue
        .filter(|_| focus.transition == FocusTransition::TransportRescue)
    else {
        return;
    };
    if let Some(feedback) = pending.iter_mut().find_map(as_feedback) {
        feedback.record(rescue);
        return;
    }
    let mut feedback = TransportRescueFeedback::default();
    feedback.record(rescue);
    pending.push_back(DeliveryCommand::RescueFeedback(feedback));
}

fn as_feedback(command: &mut DeliveryCommand) -> Option<&mut TransportRescueFeedback> {
    match command {
        DeliveryCommand::RescueFeedback(feedback) => Some(feedback),
        _ => None,
    }
}
