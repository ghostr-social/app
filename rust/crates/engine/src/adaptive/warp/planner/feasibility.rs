use super::reserve_progress;
use super::types::{ReserveProgressPolicy, SemanticDecision, WarpPlannerConfig, WarpPlannerInput};
use crate::adaptive::{
    ActionKind, ActionNode, HardBudget, PlannerCapability, PlayerPreparation, ReserveConstraint,
    SemanticCandidate, SemanticGuardrail,
};
use std::collections::BTreeSet;

mod hard_budget;
mod rescue;
mod reserve;

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
    let rescue = rescue::select(&rescue::RescueInputs {
        input,
        frontier,
        semantic: &semantic,
        config,
        budget: &budget,
    });
    let (budget, reserve) = reserve::protect(input, budget, rescue.as_ref());
    let nodes = frontier
        .iter()
        .filter(|node| {
            node.post == input.snapshot.playback.current || semantically_admissible(node, &semantic)
        })
        .filter(|node| request_admitted(input, node))
        .filter(|node| budget.allows_node(node))
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
    let window = semantic_window(input);
    let guardrail = SemanticGuardrail::new(config.semantic_top_k, config.semantic_epsilon_micros);
    let reserve = (config.reserve_progress_policy == ReserveProgressPolicy::OrderedReadiness)
        .then(|| reserve_progress::target_post(input.snapshot, input.base))
        .flatten();
    window
        .iter()
        .map(|candidate| SemanticDecision {
            post: candidate.post.clone(),
            admission: if reserve == Some(&candidate.post) {
                guardrail.admit_reserve(candidate, &window)
            } else {
                guardrail.admit(candidate, &window)
            },
        })
        .collect()
}

fn semantic_window(input: &WarpPlannerInput<'_>) -> Vec<SemanticCandidate> {
    let progressive = input
        .snapshot
        .candidates
        .iter()
        .filter_map(|candidate| progressive_semantic(input, candidate));
    let hls = input
        .snapshot
        .hls_candidates
        .iter()
        .filter_map(|candidate| hls_semantic(input, candidate));
    let mut ranked: Vec<_> = progressive.chain(hls).collect();
    ranked.sort_by_key(|item| item.0);
    ranked.into_iter().map(|item| item.1).collect()
}

fn progressive_semantic(
    input: &WarpPlannerInput<'_>,
    candidate: &crate::adaptive::CandidateSnapshot,
) -> Option<(i32, SemanticCandidate)> {
    let context = input.context.candidate(&candidate.post)?;
    if candidate.feed_offset.value() < 0
        || (candidate.feed_offset.value() == 0 && ready(candidate, context.capability))
    {
        return None;
    }
    progressive_actionable(candidate, context).then(|| {
        (
            candidate.feed_offset.value(),
            SemanticCandidate::new(
                candidate.post.clone(),
                context.semantic,
                ready(candidate, context.capability),
            ),
        )
    })
}

fn hls_semantic(
    input: &WarpPlannerInput<'_>,
    candidate: &crate::adaptive::HlsCandidateSnapshot,
) -> Option<(i32, SemanticCandidate)> {
    let context = input.context.candidate(&candidate.post)?;
    if candidate.feed_offset.value() < 0
        || (candidate.feed_offset.value() == 0 && candidate.player_ready())
    {
        return None;
    }
    (!candidate.player_ready()).then(|| {
        (
            candidate.feed_offset.value(),
            SemanticCandidate::new(candidate.post.clone(), context.semantic, false),
        )
    })
}

fn progressive_actionable(
    candidate: &crate::adaptive::CandidateSnapshot,
    context: crate::adaptive::PlannerCandidateContext,
) -> bool {
    candidate.retrieval_eligible
        && (candidate.needs_bootstrap()
            || !crate::adaptive::ranges::missing(candidate).is_empty()
            || !candidate.in_flight.is_empty()
            || context.capability.required_transform().is_some())
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
    matches!(
        node.kind,
        ActionKind::Cancel(_) | ActionKind::Promote { .. }
    ) || admissible(&node.post, semantic)
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
