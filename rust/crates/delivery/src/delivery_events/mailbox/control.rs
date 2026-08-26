use crate::delivery_events::DeliveryCommand;
use std::collections::VecDeque;

pub(super) fn replace(pending: &mut VecDeque<DeliveryCommand>, mut command: DeliveryCommand) {
    if let Some(index) = pending.iter().position(|old| same_kind(old, &command)) {
        if !supersedes(&command, &pending[index]) {
            return;
        }
        let previous = pending.remove(index).expect("located command");
        inherit_focus_lineage(&mut command, &previous);
    }
    pending.push_back(command);
}

fn inherit_focus_lineage(new: &mut DeliveryCommand, previous: &DeliveryCommand) {
    let (DeliveryCommand::Focus(new), DeliveryCommand::Focus(previous)) = (new, previous) else {
        return;
    };
    if new.current_post().is_some() && new.current_post() == previous.current_post() {
        new.generation = new.generation.covering(previous.generation);
    }
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
