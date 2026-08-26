use super::{ActionKind, ActionNode};
use crate::ActionId;

impl ActionNode {
    pub(in crate::adaptive::warp) fn conflicts(&self, other: &Self) -> bool {
        if same_control_target(&self.kind, &other.kind) {
            return true;
        }
        self.post == other.post
            && (whole_fetch(&self.kind)
                || whole_fetch(&other.kind)
                || same_transfer_target(&self.kind, &other.kind))
    }

    pub(in crate::adaptive::warp) fn conflicts_with(&self, selected: &[Self]) -> bool {
        selected
            .iter()
            .any(|chosen| !self.requires.contains(&chosen.id) && chosen.conflicts(self))
    }

    pub(in crate::adaptive::warp) fn path_is_viable(path: &[Self]) -> bool {
        path.iter()
            .enumerate()
            .all(|(index, node)| node.can_follow(&path[..index]))
    }

    fn can_follow(&self, selected: &[Self]) -> bool {
        self.requires
            .iter()
            .all(|id| selected.iter().any(|chosen| chosen.id == *id))
            && !self.conflicts_with(selected)
    }
}

fn whole_fetch(kind: &ActionKind) -> bool {
    matches!(kind, ActionKind::FetchWhole { .. })
}

fn same_transfer_target(left: &ActionKind, right: &ActionKind) -> bool {
    match (left, right) {
        (ActionKind::Prefix(left), ActionKind::Prefix(right))
        | (ActionKind::Tail(left), ActionKind::Tail(right))
        | (ActionKind::FetchRange(left), ActionKind::FetchRange(right))
        | (ActionKind::CacheUpgrade(left), ActionKind::CacheUpgrade(right)) => left == right,
        _ => false,
    }
}

fn same_control_target(left: &ActionKind, right: &ActionKind) -> bool {
    control_target(left)
        .zip(control_target(right))
        .is_some_and(|(left, right)| left == right)
}

fn control_target(kind: &ActionKind) -> Option<ActionId> {
    match kind {
        ActionKind::Promote { active, .. } | ActionKind::Cancel(active) => Some(*active),
        ActionKind::Hedge { primary, .. } => Some(*primary),
        _ => None,
    }
}
