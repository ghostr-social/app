use super::ActionNode;
use crate::adaptive::EpsilonBuckets;
use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionFrontier {
    pub(crate) retained: Vec<ActionNode>,
    pub(crate) pruned_ids: Vec<u16>,
}

impl ActionFrontier {
    pub(crate) fn prune(mut actions: Vec<ActionNode>, epsilon: EpsilonBuckets) -> Self {
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
        && equivalent_effects(left, right)
        && left.forecast == right.forecast
        && left.resources.no_more_than(right.resources)
        && left.resources != right.resources
}

// Forecast summaries cannot prove stochastic dominance (WARP §6.4).
// In particular, equal coverage estimates do not identify the same bytes.
fn equivalent_effects(left: &ActionNode, right: &ActionNode) -> bool {
    (
        &left.post,
        &left.kind,
        &left.origin,
        &left.requires,
        left.value,
    ) == (
        &right.post,
        &right.kind,
        &right.origin,
        &right.requires,
        right.value,
    ) && equivalent_admission(left, right)
}

fn equivalent_admission(left: &ActionNode, right: &ActionNode) -> bool {
    left.request_profile() == right.request_profile()
        && left.origin_admission_intent() == right.origin_admission_intent()
        && left.resource_authority() == right.resource_authority()
        && left.request() == right.request()
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
    equivalent_effects(left, right)
        && equivalent_resource_interactions(left, right)
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
        && left
            .forecast
            .quality_gain_micros
            .abs_diff(right.forecast.quality_gain_micros)
            <= epsilon.quality_micros()
}

fn equivalent_resource_interactions(left: &ActionNode, right: &ActionNode) -> bool {
    left.resources.storage_bytes == right.resources.storage_bytes
        && left.resources.cpu_ms == right.resources.cpu_ms
        && left.resources.requests == right.resources.requests
}
