use super::{CandidateRetrievalLadder, GeneratedAction, PlannerContext};
use crate::adaptive::{
    ActionKind, CandidateSnapshot, CompletionTimes, DeadlineReadiness, PlanMetrics, PlannerQuality,
    PreviewAvailability, ResourceCost, RetrievalLadder, RetrievalPlan, RetrievalRung,
};
use std::collections::BTreeMap;

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
    let mut plans = vec![metadata_plan(candidate)];
    if let Some(preview) = preview_plan(candidate, candidate_context.preview) {
        plans.push(preview);
    }
    for action in actions
        .iter()
        .filter(|item| item.node.post == candidate.post)
    {
        if let Some(plan) = action_plan(
            snapshot,
            candidate,
            action,
            by_id,
            candidate_context.quality,
        ) {
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

fn metadata_plan(candidate: &CandidateSnapshot) -> RetrievalPlan {
    RetrievalPlan::new(
        format!("{}:metadata", candidate.post.as_str()),
        RetrievalRung::Metadata,
        PlanMetrics::new(0, CompletionTimes::new(0, 0, 0, 0), Vec::new(), 0)
            .with_confidence(
                candidate.evidence.confidence.readiness.basis_points(),
                candidate.evidence.confidence.integrity.basis_points(),
            )
            .with_size_bounds(candidate.evidence.size.lower, candidate.evidence.size.upper),
    )
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
    quality: PlannerQuality,
) -> Option<RetrievalPlan> {
    let rung = rung(&action.node.kind)?;
    let path = dependency_path(action, by_id);
    let metrics = metrics(snapshot, candidate, &path, quality);
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

fn rung(kind: &ActionKind) -> Option<RetrievalRung> {
    match kind {
        ActionKind::Head => Some(RetrievalRung::Metadata),
        ActionKind::Prefix(_) | ActionKind::Tail(_) => Some(RetrievalRung::FirstFrame),
        ActionKind::FetchRange(_) | ActionKind::Hedge { .. } => Some(RetrievalRung::ReadyPlayback),
        ActionKind::FetchWhole { .. }
        | ActionKind::Promote { .. }
        | ActionKind::CacheUpgrade(_) => Some(RetrievalRung::Complete),
        ActionKind::Transform(crate::adaptive::TransformKind::Transcode) => {
            Some(RetrievalRung::Transcoded)
        }
        ActionKind::Transform(_) => Some(RetrievalRung::Remuxed),
        ActionKind::Cancel(_) => None,
    }
}

fn metrics(
    snapshot: &crate::adaptive::PlayabilitySnapshot,
    candidate: &CandidateSnapshot,
    path: &[&GeneratedAction],
    quality: PlannerQuality,
) -> PlanMetrics {
    let resources = path.iter().fold(ResourceCost::default(), |sum, item| {
        add(sum, item.node.resources)
    });
    let timing = path
        .iter()
        .fold(CompletionTimes::new(0, 0, 0, 0), |sum, item| {
            add_times(sum, item.node.forecast.completion)
        });
    let success = path.iter().fold(10_000_u16, |value, item| {
        ((u32::from(value) * u32::from(item.node.forecast.success_bps)) / 10_000) as u16
    });
    let ready: u64 = path
        .iter()
        .map(|item| item.node.forecast.ready_playback_ms)
        .sum();
    let reach = (candidate.view_probability.value() * 10_000.0).round() as u64;
    let mut metrics = PlanMetrics::new(
        resources.network_bytes,
        timing,
        deadlines(snapshot.commitment_ms, timing, success),
        ready,
    )
    .with_resources(resources)
    .with_unused_bytes(resources.network_bytes.saturating_mul(10_000 - reach) / 10_000)
    .with_storage_byte_ms(
        resources
            .storage_bytes
            .saturating_mul(snapshot.commitment_ms),
    )
    .with_confidence(
        candidate.evidence.confidence.readiness.basis_points(),
        candidate.evidence.confidence.integrity.basis_points(),
    )
    .with_size_bounds(candidate.evidence.size.lower, candidate.evidence.size.upper)
    .with_cache_value(cache_value(path));
    metrics.ready_coverage_ms = ready.saturating_mul(u64::from(success)) / 10_000;
    apply_quality(metrics, quality)
}

fn apply_quality(metrics: PlanMetrics, quality: PlannerQuality) -> PlanMetrics {
    match quality {
        PlannerQuality::Unavailable => metrics.with_quality(0, 0, 10_000),
        PlannerQuality::Estimated {
            expected_micros,
            lower_micros,
            uncertainty_bps,
        } => metrics.with_quality(expected_micros, lower_micros, uncertainty_bps),
    }
}

fn deadlines(deadline: u64, timing: CompletionTimes, success: u16) -> Vec<DeadlineReadiness> {
    let probability = if timing.p95_ms <= deadline {
        success.saturating_mul(95) / 100
    } else if timing.expected_ms <= deadline {
        success / 2
    } else {
        0
    };
    vec![DeadlineReadiness::new(deadline, probability)]
}

fn add(left: ResourceCost, right: ResourceCost) -> ResourceCost {
    ResourceCost::new(
        left.network_bytes.saturating_add(right.network_bytes),
        left.storage_bytes.saturating_add(right.storage_bytes),
        left.cpu_ms.saturating_add(right.cpu_ms),
        left.requests.saturating_add(right.requests),
    )
}

fn add_times(left: CompletionTimes, right: CompletionTimes) -> CompletionTimes {
    CompletionTimes::new(
        left.expected_ms.saturating_add(right.expected_ms),
        left.p95_ms.saturating_add(right.p95_ms),
        left.p99_ms.saturating_add(right.p99_ms),
        left.cvar_ms.saturating_add(right.cvar_ms),
    )
}

fn cache_value(path: &[&GeneratedAction]) -> u64 {
    path.iter()
        .map(|item| item.node.value.cache_gain_micros.max(0) as u64)
        .sum()
}

fn unsupported_plan(candidate: &CandidateSnapshot) -> RetrievalPlan {
    RetrievalPlan::new(
        format!("{}:unsupported", candidate.post.as_str()),
        RetrievalRung::Unsupported,
        PlanMetrics::new(0, CompletionTimes::new(0, 0, 0, 0), Vec::new(), 0),
    )
}
