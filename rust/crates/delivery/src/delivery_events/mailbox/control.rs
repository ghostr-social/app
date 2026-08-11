use crate::delivery_events::DeliveryCommand;
use std::collections::VecDeque;

pub(super) fn replace(pending: &mut VecDeque<DeliveryCommand>, command: DeliveryCommand) {
    if let Some(index) = pending.iter().position(|old| same_kind(old, &command)) {
        if !supersedes(&command, &pending[index]) {
            return;
        }
        pending.remove(index);
    }
    pending.push_back(command);
}

fn same_kind(left: &DeliveryCommand, right: &DeliveryCommand) -> bool {
    use DeliveryCommand::{Config, Focus, Playback, Prioritize};
    matches!(
        (left, right),
        (Prioritize(_), Prioritize(_))
            | (Focus(_), Focus(_))
            | (Playback(_), Playback(_))
            | (Config(_), Config(_))
    )
}

fn supersedes(new: &DeliveryCommand, old: &DeliveryCommand) -> bool {
    let (DeliveryCommand::Playback(new), DeliveryCommand::Playback(old)) = (new, old) else {
        return true;
    };
    let generation = new.session.generation().cmp(&old.session.generation());
    generation.is_gt() || (generation.is_eq() && new.sequence > old.sequence)
}
