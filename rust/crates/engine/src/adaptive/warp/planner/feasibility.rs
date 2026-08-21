use super::types::{ReserveConstraint, SemanticDecision, WarpPlannerConfig, WarpPlannerInput};
use crate::adaptive::{
    ActionKind, ActionNode, ControlMode, HardBudget, PlannerCapability, PlayerPreparation,
    ResourceCost, SemanticCandidate, SemanticGuardrail,
};
use std::collections::BTreeSet;

mod hard_budget;
mod rescue;

#[cfg(test)]
#[path = "feasibility/tests/mod.rs"]
mod tests;

pub(super) struct FeasibleActions {
    pub nodes: Vec<ActionNode>,
    pub budget: HardBudget,
    pub reserve: ReserveConstraint,
    pub semantic: Vec<SemanticDecision>,
}

pub(super) fn apply(
    input: &WarpPlannerInput<'_>,
    frontier: &[ActionNode],
    config: &WarpPlannerConfig,
    network_bytes: u64,
) -> FeasibleActions {
    let budget = hard_budget::build(input, network_bytes);
    let semantic = semantic_decisions(input, config);
    let rescue = rescue::select(rescue::RescueInputs {
        input,
        frontier,
        semantic: &semantic,
        config,
        budget: &budget,
    });
    let (budget, reserve) = protect_rescue(input.base.mode, budget, rescue.as_ref());
    let nodes = frontier
        .iter()
        .filter(|node| semantically_admissible(node, &semantic))
        .filter(|node| request_admitted(input, node))
        .filter(|node| budget.allows(&node.resources, node.request_authority()))
        .filter(|node| reserve.degraded || budget.allows_action(node))
        .cloned()
        .collect();
    FeasibleActions {
        nodes: with_satisfied_dependencies(nodes),
        budget,
        reserve,
        semantic,
    }
}

fn request_admitted(input: &WarpPlannerInput<'_>, node: &ActionNode) -> bool {
    node.resources.requests == 0 || input.context.request_admits(node)
}

fn semantic_decisions(
    input: &WarpPlannerInput<'_>,
    config: &WarpPlannerConfig,
) -> Vec<SemanticDecision> {
    let window: Vec<_> = input
        .snapshot
        .candidates
        .iter()
        .filter_map(|candidate| {
            let context = input.context.candidate(&candidate.post)?;
            Some(SemanticCandidate::new(
                candidate.post.clone(),
                context.semantic,
                ready(candidate, context.capability),
            ))
        })
        .collect();
    let guardrail = SemanticGuardrail::new(config.semantic_top_k, config.semantic_epsilon_micros);
    window
        .iter()
        .map(|candidate| SemanticDecision {
            post: candidate.post.clone(),
            admission: guardrail.admit(candidate, &window),
        })
        .collect()
}

fn ready(candidate: &crate::adaptive::CandidateSnapshot, capability: PlannerCapability) -> bool {
    if capability.blocks_direct_playback() {
        return false;
    }
    if candidate.finalized || candidate.player_preparation == PlayerPreparation::FirstFrameRendered
    {
        return true;
    }
    startup_ready(candidate)
}

fn startup_ready(candidate: &crate::adaptive::CandidateSnapshot) -> bool {
    let Some(startup) = candidate.startup.as_ref() else {
        return false;
    };
    startup.ranges().iter().all(|range| {
        candidate
            .present
            .iter()
            .any(|present| present.start <= range.start && present.end >= range.end)
    })
}

fn admissible(post: &crate::PostId, semantic: &[SemanticDecision]) -> bool {
    semantic
        .iter()
        .find(|item| &item.post == post)
        .is_some_and(|item| item.admission.admissible)
}

fn semantically_admissible(node: &ActionNode, semantic: &[SemanticDecision]) -> bool {
    matches!(node.kind, ActionKind::Cancel(_)) || admissible(&node.post, semantic)
}

fn protect_rescue(
    mode: ControlMode,
    budget: HardBudget,
    rescue: Option<&rescue::RescuePlan>,
) -> (HardBudget, ReserveConstraint) {
    if mode == ControlMode::Normal {
        return (budget, ReserveConstraint::default());
    }
    let Some(plan) = rescue else {
        return (budget, degraded_reserve());
    };
    match budget.clone().protect(&plan.steps) {
        Some(protected) => (protected, reserved(plan.cost)),
        None => (budget, degraded_reserve()),
    }
}

fn reserved(cost: ResourceCost) -> ReserveConstraint {
    ReserveConstraint {
        reserved_request_slots: cost.requests,
        reserved_network_bytes: cost.network_bytes,
        degraded: false,
    }
}

fn degraded_reserve() -> ReserveConstraint {
    ReserveConstraint {
        degraded: true,
        ..ReserveConstraint::default()
    }
}

fn with_satisfied_dependencies(mut nodes: Vec<ActionNode>) -> Vec<ActionNode> {
    loop {
        let ids: BTreeSet<_> = nodes.iter().map(|node| node.id).collect();
        let before = nodes.len();
        nodes.retain(|node| node.requires.iter().all(|id| ids.contains(id)));
        if nodes.len() == before {
            return nodes;
        }
    }
}
