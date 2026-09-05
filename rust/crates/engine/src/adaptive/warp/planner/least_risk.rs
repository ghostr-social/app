use crate::adaptive::{ActionNode, PrunedSearchPlan, RetainedSearchPlan};
use crate::adaptive::{ResourcePrices, SearchDecision, SearchPruneReason};
use std::collections::BTreeSet;

#[cfg(test)]
#[path = "least_risk/dependency_test.rs"]
mod dependency_test;

pub(super) fn choose(nodes: &[ActionNode], preferred: &[u16]) -> SearchDecision {
    choose_with_reason(nodes, preferred, SearchPruneReason::ReserveUnderflow)
}

pub(super) fn choose_with_reason(
    nodes: &[ActionNode],
    preferred: &[u16],
    reason: SearchPruneReason,
) -> SearchDecision {
    let action = select(nodes, preferred);
    let chosen_plan = action.as_ref().map(plan);
    SearchDecision {
        committed_actions: u8::from(action.is_some()),
        action,
        chosen_plan: chosen_plan.clone(),
        used_greedy_fallback: true,
        retained_plans: chosen_plan.into_iter().collect(),
        pruned_plans: vec![PrunedSearchPlan {
            action_ids: Vec::new(),
            reason,
        }],
        pruned_plan_events_total: 1,
        pruned_plan_sample_truncated: false,
    }
}

fn select(nodes: &[ActionNode], preferred: &[u16]) -> Option<ActionNode> {
    preferred_root(nodes, preferred)
        .or_else(|| least_risk_root(nodes, true))
        .or_else(|| least_risk_root(nodes, false))
}

fn preferred_root(nodes: &[ActionNode], terminals: &[u16]) -> Option<ActionNode> {
    nodes
        .iter()
        .filter(|node| node.requires.is_empty())
        .filter(|node| terminals.iter().any(|id| reaches(nodes, *id, node.id)))
        .min_by_key(risk_key)
        .cloned()
}

fn reaches(nodes: &[ActionNode], terminal: u16, root: u16) -> bool {
    let mut pending = vec![terminal];
    let mut seen = BTreeSet::new();
    while let Some(id) = pending.pop() {
        if id == root {
            return true;
        }
        if seen.insert(id) {
            let dependencies = nodes
                .iter()
                .find(|node| node.id == id)
                .into_iter()
                .flat_map(|node| node.requires.iter().copied());
            pending.extend(dependencies);
        }
    }
    false
}

fn least_risk_root(nodes: &[ActionNode], require_ready_gain: bool) -> Option<ActionNode> {
    nodes
        .iter()
        .filter(|node| node.requires.is_empty())
        .filter(|node| !require_ready_gain || node.forecast.ready_playback_ms > 0)
        .min_by_key(risk_key)
        .cloned()
}

fn risk_key(node: &&ActionNode) -> (u64, core::cmp::Reverse<u16>, u16) {
    (
        node.forecast.completion.p99_ms,
        core::cmp::Reverse(node.forecast.success_bps),
        node.id,
    )
}

fn plan(node: &ActionNode) -> RetainedSearchPlan {
    RetainedSearchPlan {
        action_ids: vec![node.id],
        score_micros: node.value.total(node.resources, ResourcePrices::default()),
    }
}
