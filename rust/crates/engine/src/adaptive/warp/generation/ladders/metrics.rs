use super::super::GeneratedAction;
use crate::adaptive::{
    CandidateSnapshot, CompletionTimes, PlanMetrics, PlannerCandidateContext, PlannerQuality,
    ResourceCost,
};

pub(super) fn build(
    snapshot: &crate::adaptive::PlayabilitySnapshot,
    candidate: &CandidateSnapshot,
    path: &[&GeneratedAction],
    evidence: PlannerCandidateContext,
) -> PlanMetrics {
    let resources = resources(path);
    let timing = completion(path);
    let success = success(path);
    let ready = ready_ms(path);
    let reach = (candidate.view_probability.value() * 10_000.0).round() as u64;
    let mut metrics = PlanMetrics::new(
        resources.network_bytes,
        timing,
        super::deadline::readiness(snapshot.commitment_ms, timing, success, evidence.watch),
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
    apply_quality(metrics, evidence.quality)
}

fn resources(path: &[&GeneratedAction]) -> ResourceCost {
    path.iter().fold(ResourceCost::default(), |sum, item| {
        add(sum, item.node.resources)
    })
}

fn completion(path: &[&GeneratedAction]) -> CompletionTimes {
    path.iter()
        .fold(CompletionTimes::new(0, 0, 0, 0), |sum, item| {
            add_times(sum, item.node.forecast.completion)
        })
}

fn success(path: &[&GeneratedAction]) -> u16 {
    path.iter().fold(10_000, |value, item| {
        ((u32::from(value) * u32::from(item.node.forecast.success_bps)) / 10_000) as u16
    })
}

fn ready_ms(path: &[&GeneratedAction]) -> u64 {
    path.iter()
        .map(|item| item.node.forecast.ready_playback_ms)
        .sum()
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
