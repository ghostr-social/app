use crate::adaptive::{
    CompletionTimes, DeadlineReadiness, EpsilonBuckets, PlanMetrics, ResourceCost, RetrievalLadder,
    RetrievalPlan, RetrievalRung,
};

fn metrics(bytes: u64, p95: u64, coverage: u64, requests: u16) -> PlanMetrics {
    PlanMetrics::new(
        bytes,
        CompletionTimes::new(p95 / 2, p95, p95 * 2, p95 * 2),
        vec![DeadlineReadiness::new(3_000, 9_500)],
        coverage,
    )
    .with_resources(ResourceCost::new(bytes, bytes, 0, requests))
    .with_unused_bytes(bytes / 10)
    .with_storage_byte_ms(bytes * 2_000)
    .with_confidence(9_000, 8_000)
    .with_quality(700_000, 650_000, 1_000)
    .with_size_bounds(Some(bytes), Some(bytes))
    .with_cache_value(50_000)
}

#[test]
fn exact_pareto_removes_only_a_plan_worse_in_every_monotone_dimension() {
    let efficient = RetrievalPlan::new(
        "prefix",
        RetrievalRung::FirstFrame,
        metrics(64_000, 80, 2_000, 1),
    );
    let dominated = RetrievalPlan::new(
        "large-probe",
        RetrievalRung::FirstFrame,
        metrics(128_000, 120, 2_000, 2),
    );
    let broader = RetrievalPlan::new(
        "whole",
        RetrievalRung::Complete,
        metrics(800_000, 400, 60_000, 1),
    );

    let ladder = RetrievalLadder::prune(
        vec![dominated, broader.clone(), efficient.clone()],
        EpsilonBuckets::disabled(),
    );

    assert_eq!(ladder.plans(), &[efficient, broader]);
}

#[test]
fn configurable_epsilon_buckets_merge_near_duplicates_deterministically() {
    let first = RetrievalPlan::new(
        "a",
        RetrievalRung::FirstFrame,
        metrics(64_000, 100, 2_000, 1),
    );
    let near = RetrievalPlan::new(
        "b",
        RetrievalRung::FirstFrame,
        metrics(70_000, 115, 2_000, 1),
    );
    let config = EpsilonBuckets::new(20, 16_384, 100, 100);

    let ladder = RetrievalLadder::prune(vec![near, first.clone()], config);

    assert_eq!(ladder.plans(), &[first]);
}
