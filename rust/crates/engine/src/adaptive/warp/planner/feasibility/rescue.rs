use super::super::types::{SemanticDecision, WarpPlannerConfig, WarpPlannerInput};
use crate::adaptive::{
    ActionNode, ControlMode, HardBudget, RescueChanceEvidence, RescueTimingQuantile, ResourceCost,
};
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
    pub chance: RescueChanceEvidence,
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
    if !steps.iter().all(|node| admitted_for_rescue(inputs, node)) {
        return None;
    }
    if !steps
        .iter()
        .all(|node| super::request_admitted(inputs.input, node))
    {
        return None;
    }
    let chance = chance_evidence(inputs, &steps)?;
    let cost = HardBudget::path_cost(&steps)?;
    inputs.budget.clone().protect(&steps)?;
    Some(RescuePlan {
        steps,
        cost,
        chance,
    })
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

fn chance_evidence(
    inputs: &RescueInputs<'_>,
    steps: &[ActionNode],
) -> Option<RescueChanceEvidence> {
    // Union-bound transport and quantile failures without assuming independence.
    let threshold_bps = threshold(inputs.input.base.mode, inputs.config);
    let transport_failure = transport_failure(steps);
    let allowed = 10_000_u32.saturating_sub(u32::from(threshold_bps));
    let timing = timing_evidence(steps, allowed.checked_sub(transport_failure)?)?;
    let deadline_ms = inputs.input.snapshot.commitment_ms;
    (timing.completion_ms <= deadline_ms).then_some(RescueChanceEvidence {
        deadline_ms,
        threshold_bps,
        achieved_success_bps: success_bps(transport_failure + timing.failure_bps),
        transport_success_bps: success_bps(transport_failure),
        timing_quantile: timing.quantile,
        timing_completion_ms: timing.completion_ms,
    })
}

fn timing_evidence(steps: &[ActionNode], allowed_failure: u32) -> Option<TimingEvidence> {
    let count = u32::try_from(steps.len()).ok()?;
    let (quantile, failure_per_step) = if count.saturating_mul(P95_FAILURE_BPS) <= allowed_failure {
        (RescueTimingQuantile::P95, P95_FAILURE_BPS)
    } else if count.saturating_mul(P99_FAILURE_BPS) <= allowed_failure {
        (RescueTimingQuantile::P99, P99_FAILURE_BPS)
    } else {
        return None;
    };
    Some(TimingEvidence {
        quantile,
        failure_bps: count.saturating_mul(failure_per_step),
        completion_ms: timing_completion(steps, quantile),
    })
}

fn transport_failure(steps: &[ActionNode]) -> u32 {
    steps.iter().fold(0_u32, |failure, node| {
        failure.saturating_add(u32::from(
            10_000_u16.saturating_sub(node.forecast.success_bps),
        ))
    })
}

fn timing_completion(steps: &[ActionNode], quantile: RescueTimingQuantile) -> u64 {
    steps.iter().fold(0_u64, |total, node| {
        total.saturating_add(match quantile {
            RescueTimingQuantile::P95 => node.forecast.completion.p95_ms,
            RescueTimingQuantile::P99 => node.forecast.completion.p99_ms,
        })
    })
}

fn success_bps(failure_bps: u32) -> u16 {
    10_000_u32.saturating_sub(failure_bps).min(10_000) as u16
}

#[derive(Clone, Copy)]
struct TimingEvidence {
    quantile: RescueTimingQuantile,
    failure_bps: u32,
    completion_ms: u64,
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

fn admitted_for_rescue(inputs: &RescueInputs<'_>, node: &ActionNode) -> bool {
    node.post == inputs.input.snapshot.playback.current || admitted(node, inputs.semantic)
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
