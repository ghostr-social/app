use super::super::types::{SemanticDecision, WarpPlannerConfig, WarpPlannerInput};
use crate::adaptive::{ActionNode, ControlMode, HardBudget, ResourceCost};
use std::collections::BTreeSet;

const P95_FAILURE_BPS: u32 = 500;
const P99_FAILURE_BPS: u32 = 100;

#[cfg(test)]
#[path = "rescue/tests/mod.rs"]
mod tests;

pub(super) struct RescueInputs<'a> {
    pub input: &'a WarpPlannerInput<'a>,
    pub frontier: &'a [ActionNode],
    pub semantic: &'a [SemanticDecision],
    pub config: &'a WarpPlannerConfig,
    pub budget: &'a HardBudget,
}

pub(super) struct RescuePlan {
    pub steps: Vec<ActionNode>,
    pub cost: ResourceCost,
}

pub(super) fn select(_inputs: RescueInputs<'_>) -> Option<RescuePlan> {
    if _inputs.input.base.mode == ControlMode::Normal {
        return None;
    }
    _inputs
        .frontier
        .iter()
        .filter(|node| node.forecast.ready_playback_ms > 0)
        .filter_map(|node| candidate(&_inputs, node))
        .min_by_key(plan_key)
}

fn candidate(inputs: &RescueInputs<'_>, terminal: &ActionNode) -> Option<RescuePlan> {
    let steps = dependency_path(inputs.frontier, terminal.id)?;
    if !steps.iter().all(|node| admitted(node, inputs.semantic)) {
        return None;
    }
    if !chance_feasible(inputs, &steps) {
        return None;
    }
    let cost = HardBudget::path_cost(&steps)?;
    inputs.budget.clone().protect(&steps)?;
    Some(RescuePlan { steps, cost })
}

fn dependency_path(frontier: &[ActionNode], terminal: u16) -> Option<Vec<ActionNode>> {
    let mut path = Vec::new();
    append_dependencies(frontier, terminal, &mut path, &mut BTreeSet::new())?;
    Some(path)
}

fn append_dependencies(
    frontier: &[ActionNode],
    id: u16,
    path: &mut Vec<ActionNode>,
    visiting: &mut BTreeSet<u16>,
) -> Option<()> {
    if path.iter().any(|node| node.id == id) {
        return Some(());
    }
    if !visiting.insert(id) {
        return None;
    }
    let node = frontier.iter().find(|node| node.id == id)?;
    for dependency in &node.requires {
        append_dependencies(frontier, *dependency, path, visiting)?;
    }
    visiting.remove(&id);
    path.push(node.clone());
    Some(())
}

fn chance_feasible(inputs: &RescueInputs<'_>, steps: &[ActionNode]) -> bool {
    // Union-bound transport and quantile failures without assuming independence.
    let required = threshold(inputs.input.base.mode, inputs.config);
    let allowed_failure = 10_000_u32.saturating_sub(u32::from(required));
    let Some(timing_allowance) = allowed_failure.checked_sub(transport_failure(steps)) else {
        return false;
    };
    timing_completion(steps, timing_allowance)
        .is_some_and(|completion| completion <= inputs.input.snapshot.commitment_ms)
}

fn timing_completion(steps: &[ActionNode], allowed_failure: u32) -> Option<u64> {
    let count = u32::try_from(steps.len()).ok()?;
    let quantile = if count.saturating_mul(P95_FAILURE_BPS) <= allowed_failure {
        TimingQuantile::P95
    } else if count.saturating_mul(P99_FAILURE_BPS) <= allowed_failure {
        TimingQuantile::P99
    } else {
        return None;
    };
    Some(steps.iter().fold(0_u64, |total, node| {
        total.saturating_add(quantile.completion(node))
    }))
}

fn transport_failure(steps: &[ActionNode]) -> u32 {
    steps.iter().fold(0_u32, |failure, node| {
        failure.saturating_add(u32::from(
            10_000_u16.saturating_sub(node.forecast.success_bps),
        ))
    })
}

#[derive(Clone, Copy)]
enum TimingQuantile {
    P95,
    P99,
}

impl TimingQuantile {
    fn completion(self, node: &ActionNode) -> u64 {
        match self {
            Self::P95 => node.forecast.completion.p95_ms,
            Self::P99 => node.forecast.completion.p99_ms,
        }
    }
}

fn threshold(mode: ControlMode, config: &WarpPlannerConfig) -> u16 {
    match mode {
        ControlMode::Emergency => config.emergency_rescue_bps,
        ControlMode::Safety => config.safety_rescue_bps,
        ControlMode::Normal => 0,
    }
}

fn admitted(node: &ActionNode, semantic: &[SemanticDecision]) -> bool {
    semantic
        .iter()
        .find(|item| item.post == node.post)
        .is_some_and(|item| item.admission.admissible)
}

fn plan_key(plan: &RescuePlan) -> (u16, u64, u64, u64, u16) {
    let terminal = plan.steps.last().map_or(u16::MAX, |node| node.id);
    (
        plan.cost.requests,
        plan.cost.network_bytes,
        plan.cost.storage_bytes,
        plan.cost.cpu_ms,
        terminal,
    )
}
