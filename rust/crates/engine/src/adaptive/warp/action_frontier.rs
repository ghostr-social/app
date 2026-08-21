use super::{ActionKind, ActionNode, ResourcePrices};
use crate::adaptive::EpsilonBuckets;
use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionFrontier {
    pub retained: Vec<ActionNode>,
    pub pruned_ids: Vec<u16>,
}

impl ActionFrontier {
    pub fn prune(mut actions: Vec<ActionNode>, epsilon: EpsilonBuckets) -> Self {
        actions.sort_by_key(|action| action.id);
        let exact: Vec<_> = actions
            .iter()
            .filter(|action| !actions.iter().any(|other| dominates(other, action)))
            .cloned()
            .collect();
        let retained = with_dependencies(&actions, epsilon_merge(exact, epsilon));
        let ids: BTreeSet<_> = retained.iter().map(|action| action.id).collect();
        let pruned_ids = actions
            .iter()
            .filter(|action| !ids.contains(&action.id))
            .map(|action| action.id)
            .collect();
        Self {
            retained,
            pruned_ids,
        }
    }
}

fn with_dependencies(actions: &[ActionNode], mut retained: Vec<ActionNode>) -> Vec<ActionNode> {
    loop {
        let required: BTreeSet<_> = retained
            .iter()
            .flat_map(|action| action.requires.iter().copied())
            .collect();
        let missing: Vec<_> = actions
            .iter()
            .filter(|action| required.contains(&action.id))
            .filter(|action| !retained.iter().any(|item| item.id == action.id))
            .cloned()
            .collect();
        if missing.is_empty() {
            retained.sort_by_key(|action| action.id);
            return retained;
        }
        retained.extend(missing);
    }
}

fn dominates(left: &ActionNode, right: &ActionNode) -> bool {
    left.id != right.id
        && left.post == right.post
        && broader_unlock(&left.kind, &right.kind)
        && no_worse(left, right)
        && !no_worse(right, left)
}

fn no_worse(left: &ActionNode, right: &ActionNode) -> bool {
    left.forecast.ready_playback_ms >= right.forecast.ready_playback_ms
        && left.forecast.success_bps >= right.forecast.success_bps
        && left.forecast.completion.expected_ms <= right.forecast.completion.expected_ms
        && left.forecast.completion.p95_ms <= right.forecast.completion.p95_ms
        && left.forecast.completion.p99_ms <= right.forecast.completion.p99_ms
        && left.forecast.completion.cvar_ms <= right.forecast.completion.cvar_ms
        && left.resources.no_more_than(right.resources)
        && static_value(left) >= static_value(right)
}

fn static_value(action: &ActionNode) -> i64 {
    action
        .value
        .total(action.resources, ResourcePrices::default())
}

fn broader_unlock(left: &ActionKind, right: &ActionKind) -> bool {
    let (Some(left), Some(right)) = (unlock(left), unlock(right)) else {
        return std::mem::discriminant(left) == std::mem::discriminant(right);
    };
    left >= right
}

fn unlock(action: &ActionKind) -> Option<u8> {
    match action {
        ActionKind::Head => Some(0),
        ActionKind::Prefix(_) | ActionKind::Tail(_) => Some(1),
        ActionKind::FetchRange(_) | ActionKind::Hedge { .. } => Some(2),
        ActionKind::FetchWhole { .. }
        | ActionKind::Promote { .. }
        | ActionKind::CacheUpgrade(_) => Some(3),
        ActionKind::Transform(_) => Some(4),
        ActionKind::Cancel(_) => None,
    }
}

fn epsilon_merge(actions: Vec<ActionNode>, epsilon: EpsilonBuckets) -> Vec<ActionNode> {
    if epsilon == EpsilonBuckets::disabled() {
        return actions;
    }
    let mut retained = Vec::new();
    for action in actions {
        if !retained.iter().any(|other| near(other, &action, epsilon)) {
            retained.push(action);
        }
    }
    retained
}

fn near(left: &ActionNode, right: &ActionNode, epsilon: EpsilonBuckets) -> bool {
    unlock(&left.kind) == unlock(&right.kind)
        && left.post == right.post
        && left
            .forecast
            .completion
            .p95_ms
            .abs_diff(right.forecast.completion.p95_ms)
            <= epsilon.readiness_time_ms()
        && left
            .resources
            .network_bytes
            .abs_diff(right.resources.network_bytes)
            <= epsilon.readiness_bytes()
        && left
            .forecast
            .ready_playback_ms
            .abs_diff(right.forecast.ready_playback_ms)
            <= epsilon.coverage_ms()
}
