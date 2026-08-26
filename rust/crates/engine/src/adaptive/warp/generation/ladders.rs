use super::{CandidateRetrievalLadder, GeneratedAction, PlannerContext};
use crate::adaptive::{
    ActionKind, CandidateSnapshot, CompletionTimes, PlanMetrics, PlannerCandidateContext,
    PreviewAvailability, RetrievalLadder, RetrievalPlan, RetrievalRung,
};
use std::collections::BTreeMap;

mod deadline;
mod metrics;

pub(super) fn build(
    snapshot: &crate::adaptive::PlayabilitySnapshot,
    actions: &[GeneratedAction],
    context: &PlannerContext,
) -> Vec<CandidateRetrievalLadder> {
    let by_id: BTreeMap<_, _> = actions.iter().map(|item| (item.node.id, item)).collect();
    snapshot
        .candidates
        .iter()
        .filter(|candidate| candidate.retrieval_eligible)
        .map(|candidate| candidate_ladder(snapshot, candidate, actions, &by_id, context))
        .collect()
}

fn candidate_ladder(
    snapshot: &crate::adaptive::PlayabilitySnapshot,
    candidate: &CandidateSnapshot,
    actions: &[GeneratedAction],
    by_id: &BTreeMap<u16, &GeneratedAction>,
    context: &PlannerContext,
) -> CandidateRetrievalLadder {
    let candidate_context = context
        .candidate(&candidate.post)
        .expect("planner context covers every snapshot candidate");
    let mut plans = vec![metadata_plan(candidate, candidate_context)];
    if let Some(preview) = preview_plan(candidate, candidate_context.preview) {
        plans.push(preview);
    }
    for action in actions
        .iter()
        .filter(|item| item.node.post == candidate.post)
    {
        if let Some(plan) = action_plan(snapshot, candidate, action, by_id, candidate_context) {
            plans.push(plan);
        }
    }
    if candidate_context.capability.required_transform().is_none()
        && matches!(
            candidate_context.capability,
            super::super::PlannerCapability::Reported {
                playback_supported: false,
                ..
            }
        )
    {
        plans.push(unsupported_plan(candidate));
    }
    CandidateRetrievalLadder {
        post: candidate.post.clone(),
        frontier: RetrievalLadder::prune(plans, context.epsilon),
    }
}

fn metadata_plan(
    candidate: &CandidateSnapshot,
    evidence: PlannerCandidateContext,
) -> RetrievalPlan {
    let (lower, upper) = size_bounds(candidate, evidence);
    RetrievalPlan::new(
        format!("{}:metadata", candidate.post.as_str()),
        RetrievalRung::Metadata,
        PlanMetrics::new(0, CompletionTimes::new(0, 0, 0, 0), Vec::new(), 0)
            .with_confidence(
                candidate.evidence.confidence.readiness.basis_points(),
                candidate.evidence.confidence.integrity.basis_points(),
            )
            .with_size_bounds(lower, upper),
    )
}

fn size_bounds(
    candidate: &CandidateSnapshot,
    evidence: PlannerCandidateContext,
) -> (Option<u64>, Option<u64>) {
    let observed = evidence
        .whole_body_exhaustion
        .map(|item| item.observed_bytes());
    let lower = candidate.evidence.size.lower.max(observed);
    let upper = candidate
        .evidence
        .size
        .upper
        .filter(|upper| lower.is_none_or(|lower| *upper >= lower));
    (lower, upper)
}

fn preview_plan(
    candidate: &CandidateSnapshot,
    preview: PreviewAvailability,
) -> Option<RetrievalPlan> {
    let PreviewAvailability::Ready {
        bytes,
        quality_micros,
    } = preview
    else {
        return None;
    };
    Some(RetrievalPlan::new(
        format!("{}:preview", candidate.post.as_str()),
        RetrievalRung::Preview,
        PlanMetrics::new(bytes, CompletionTimes::new(0, 0, 0, 0), Vec::new(), 0).with_quality(
            quality_micros,
            quality_micros,
            0,
        ),
    ))
}

fn action_plan(
    snapshot: &crate::adaptive::PlayabilitySnapshot,
    candidate: &CandidateSnapshot,
    action: &GeneratedAction,
    by_id: &BTreeMap<u16, &GeneratedAction>,
    evidence: PlannerCandidateContext,
) -> Option<RetrievalPlan> {
    let rung = rung(candidate, &action.node.kind)?;
    let path = dependency_path(action, by_id);
    let metrics = metrics::build(snapshot, candidate, &path, evidence);
    Some(
        RetrievalPlan::new(
            format!("{}:{}", candidate.post.as_str(), action.node.id),
            rung,
            metrics,
        )
        .with_actions(path.iter().map(|item| item.node.kind.clone()).collect()),
    )
}

fn dependency_path<'a>(
    action: &'a GeneratedAction,
    by_id: &BTreeMap<u16, &'a GeneratedAction>,
) -> Vec<&'a GeneratedAction> {
    let mut path: Vec<_> = action
        .node
        .requires
        .iter()
        .filter_map(|id| by_id.get(id).copied())
        .flat_map(|required| dependency_path(required, by_id))
        .collect();
    if !path.iter().any(|item| item.node.id == action.node.id) {
        path.push(action);
    }
    path
}

fn rung(candidate: &CandidateSnapshot, kind: &ActionKind) -> Option<RetrievalRung> {
    match kind {
        ActionKind::Head => Some(RetrievalRung::Metadata),
        ActionKind::Prefix(_) | ActionKind::Tail(_) => Some(RetrievalRung::FirstFrame),
        ActionKind::FetchRange(_) | ActionKind::Hedge { .. } => Some(RetrievalRung::ReadyPlayback),
        ActionKind::FetchWhole { .. } if candidate.total_bytes.is_none() => {
            Some(RetrievalRung::Metadata)
        }
        ActionKind::FetchWhole { .. }
        | ActionKind::Promote { .. }
        | ActionKind::CacheUpgrade(_) => Some(RetrievalRung::Complete),
        ActionKind::Transform(crate::adaptive::TransformKind::Transcode) => {
            Some(RetrievalRung::Transcoded)
        }
        ActionKind::Transform(_) => Some(RetrievalRung::Remuxed),
        ActionKind::HlsBootstrap { .. } | ActionKind::Cancel(_) => None,
    }
}

fn unsupported_plan(candidate: &CandidateSnapshot) -> RetrievalPlan {
    RetrievalPlan::new(
        format!("{}:unsupported", candidate.post.as_str()),
        RetrievalRung::Unsupported,
        PlanMetrics::new(0, CompletionTimes::new(0, 0, 0, 0), Vec::new(), 0),
    )
}
