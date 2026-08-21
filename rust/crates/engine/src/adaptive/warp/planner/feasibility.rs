use super::types::{ReserveConstraint, SemanticDecision, WarpPlannerConfig, WarpPlannerInput};
use crate::adaptive::{
    ActionNode, ControlMode, HardBudget, PlayerPreparation, ResourceCost, SemanticCandidate,
    SemanticGuardrail,
};
use std::collections::BTreeSet;

mod hard_budget;

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
    let rescue_ids = rescue_actions(input, frontier, &semantic, config);
    let reserve = reserve_constraint(input.base.mode, frontier, &rescue_ids);
    let ordinary = ordinary_budget(&budget, reserve);
    let nodes = frontier
        .iter()
        .filter(|node| admissible(&node.post, &semantic))
        .filter(|node| budget.allows(&node.resources, &node.origin))
        .filter(|node| {
            reserve.degraded
                || rescue_ids.contains(&node.id)
                || ordinary.allows(&node.resources, &node.origin)
        })
        .cloned()
        .collect();
    FeasibleActions {
        nodes: with_satisfied_dependencies(nodes),
        budget,
        reserve,
        semantic,
    }
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
                ready(candidate),
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

fn ready(candidate: &crate::adaptive::CandidateSnapshot) -> bool {
    candidate.finalized
        || candidate.player_preparation == PlayerPreparation::FirstFrameRendered
        || candidate.startup.as_ref().is_some_and(|startup| {
            startup.ranges().iter().all(|range| {
                candidate
                    .present
                    .iter()
                    .any(|present| present.start <= range.start && present.end >= range.end)
            })
        })
}

fn rescue_actions(
    input: &WarpPlannerInput<'_>,
    frontier: &[ActionNode],
    semantic: &[SemanticDecision],
    config: &WarpPlannerConfig,
) -> BTreeSet<u16> {
    let threshold = rescue_threshold(input.base.mode, config);
    let mut ids: BTreeSet<_> = frontier
        .iter()
        .filter(|node| node.forecast.ready_playback_ms > 0)
        .filter(|node| admissible(&node.post, semantic))
        .filter(|node| on_time(node, input.base.mode, input.snapshot.commitment_ms) >= threshold)
        .map(|node| node.id)
        .collect();
    let dependencies: Vec<_> = frontier
        .iter()
        .filter(|node| ids.contains(&node.id))
        .flat_map(|node| node.requires.iter().copied())
        .collect();
    ids.extend(dependencies);
    ids
}

fn admissible(post: &crate::PostId, semantic: &[SemanticDecision]) -> bool {
    semantic
        .iter()
        .find(|item| &item.post == post)
        .is_some_and(|item| item.admission.admissible)
}

fn on_time(node: &ActionNode, mode: ControlMode, deadline_ms: u64) -> u16 {
    let completion = match mode {
        ControlMode::Emergency => node.forecast.completion.p99_ms,
        ControlMode::Safety => node.forecast.completion.p95_ms,
        ControlMode::Normal => node.forecast.completion.expected_ms,
    };
    if completion <= deadline_ms {
        node.forecast.success_bps
    } else {
        0
    }
}

fn rescue_threshold(mode: ControlMode, config: &WarpPlannerConfig) -> u16 {
    match mode {
        ControlMode::Emergency => config.emergency_rescue_bps,
        ControlMode::Safety => config.safety_rescue_bps,
        ControlMode::Normal => 0,
    }
}

fn reserve_constraint(
    mode: ControlMode,
    frontier: &[ActionNode],
    rescue: &BTreeSet<u16>,
) -> ReserveConstraint {
    if mode == ControlMode::Normal {
        return ReserveConstraint::default();
    }
    let bytes = frontier
        .iter()
        .filter(|node| rescue.contains(&node.id))
        .map(|node| node.resources.network_bytes)
        .min();
    match bytes {
        Some(bytes) => ReserveConstraint {
            reserved_request_slots: 1,
            reserved_network_bytes: bytes,
            degraded: false,
        },
        None => ReserveConstraint {
            degraded: true,
            ..ReserveConstraint::default()
        },
    }
}

fn ordinary_budget(budget: &HardBudget, reserve: ReserveConstraint) -> HardBudget {
    let mut ordinary = budget.clone();
    let reserved = ResourceCost::new(
        reserve.reserved_network_bytes,
        0,
        0,
        reserve.reserved_request_slots,
    );
    ordinary.consume(&reserved, "reserved-rescue");
    ordinary
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
