use crate::delivery_events::DeliveryCommand;
use std::collections::VecDeque;

mod focus;

pub(super) fn replace(pending: &mut VecDeque<DeliveryCommand>, command: DeliveryCommand) {
    let command = match command {
        DeliveryCommand::Focus(update) => return focus::replace(pending, update),
        command => command,
    };
    if let Some(index) = pending.iter().position(|old| same_kind(old, &command)) {
        if !supersedes(&command, &pending[index]) {
            return;
        }
        pending.remove(index).expect("located command");
    }
    pending.push_back(command);
}

pub(super) fn compact_for_delivery(pending: &mut VecDeque<DeliveryCommand>) {
    focus::compact_for_delivery(pending);
}

fn same_kind(left: &DeliveryCommand, right: &DeliveryCommand) -> bool {
    use DeliveryCommand::{Config, Focus, NetworkProfile, NetworkStatus, Playback, StorageChanged};
    matches!(
        (left, right),
        (Focus(_), Focus(_))
            | (Playback(_), Playback(_))
            | (Config(_), Config(_))
            | (NetworkStatus(_), NetworkStatus(_))
            | (NetworkProfile { .. }, NetworkProfile { .. })
            | (StorageChanged, StorageChanged)
    )
}

fn supersedes(new: &DeliveryCommand, old: &DeliveryCommand) -> bool {
    if let (DeliveryCommand::NetworkStatus(new), DeliveryCommand::NetworkStatus(old)) = (new, old) {
        return new.is_fresher_than(*old);
    }
    if let (
        DeliveryCommand::NetworkProfile {
            generation: new, ..
        },
        DeliveryCommand::NetworkProfile {
            generation: old, ..
        },
    ) = (new, old)
    {
        return new > old;
    }
    let (DeliveryCommand::Playback(new), DeliveryCommand::Playback(old)) = (new, old) else {
        return true;
    };
    let generation = new.session.generation().cmp(&old.session.generation());
    generation.is_gt() || (generation.is_eq() && new.sequence > old.sequence)
}
