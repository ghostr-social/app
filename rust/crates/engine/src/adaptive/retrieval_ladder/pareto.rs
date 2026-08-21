use super::{PlanMetrics, RetrievalPlan};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct EpsilonBuckets {
    readiness_time_ms: u64,
    readiness_bytes: u64,
    coverage_ms: u64,
    quality_micros: u64,
}

impl EpsilonBuckets {
    pub const fn disabled() -> Self {
        Self {
            readiness_time_ms: 0,
            readiness_bytes: 0,
            coverage_ms: 0,
            quality_micros: 0,
        }
    }

    pub const fn new(time_ms: u64, bytes: u64, coverage_ms: u64, quality_micros: u64) -> Self {
        Self {
            readiness_time_ms: time_ms,
            readiness_bytes: bytes,
            coverage_ms,
            quality_micros,
        }
    }

    pub const fn readiness_time_ms(self) -> u64 {
        self.readiness_time_ms
    }

    pub const fn readiness_bytes(self) -> u64 {
        self.readiness_bytes
    }

    pub const fn coverage_ms(self) -> u64 {
        self.coverage_ms
    }

    pub const fn quality_micros(self) -> u64 {
        self.quality_micros
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RetrievalLadder {
    plans: Vec<RetrievalPlan>,
}

impl RetrievalLadder {
    pub fn prune(mut plans: Vec<RetrievalPlan>, epsilon: EpsilonBuckets) -> Self {
        plans.sort_by(|left, right| left.id().cmp(right.id()));
        let exact: Vec<_> = plans
            .iter()
            .filter(|plan| !plans.iter().any(|other| dominates(other, plan)))
            .cloned()
            .collect();
        Self {
            plans: merge_epsilon(exact, epsilon),
        }
    }

    pub fn plans(&self) -> &[RetrievalPlan] {
        &self.plans
    }
}

fn dominates(left: &RetrievalPlan, right: &RetrievalPlan) -> bool {
    if left.id() == right.id() || !no_worse(&left.metrics, &right.metrics) {
        return false;
    }
    strictly_better(&left.metrics, &right.metrics)
}

fn strictly_better(left: &PlanMetrics, right: &PlanMetrics) -> bool {
    !no_worse(right, left)
}

fn no_worse(left: &PlanMetrics, right: &PlanMetrics) -> bool {
    benefits(left, right) && costs(left, right) && deadlines(left, right)
}

fn benefits(left: &PlanMetrics, right: &PlanMetrics) -> bool {
    left.ready_playback_ms >= right.ready_playback_ms
        && left.ready_coverage_ms >= right.ready_coverage_ms
        && left.quality.lower_micros >= right.quality.lower_micros
        && left.streamability_bps >= right.streamability_bps
        && left.integrity_bps >= right.integrity_bps
        && left.cache_value_micros >= right.cache_value_micros
}

fn costs(left: &PlanMetrics, right: &PlanMetrics) -> bool {
    left.readiness_bytes <= right.readiness_bytes
        && left.readiness_time.expected_ms <= right.readiness_time.expected_ms
        && left.readiness_time.p95_ms <= right.readiness_time.p95_ms
        && left.readiness_time.p99_ms <= right.readiness_time.p99_ms
        && left.readiness_time.cvar_ms <= right.readiness_time.cvar_ms
        && left.resources.no_more_than(right.resources)
        && left.unused_bytes <= right.unused_bytes
        && left.storage_byte_ms <= right.storage_byte_ms
        && left.quality.uncertainty_bps <= right.quality.uncertainty_bps
        && upper_no_larger(left.size.upper, right.size.upper)
}

fn deadlines(left: &PlanMetrics, right: &PlanMetrics) -> bool {
    right.readiness_by_deadline.iter().all(|wanted| {
        left.readiness_by_deadline
            .iter()
            .find(|item| item.deadline_ms == wanted.deadline_ms)
            .is_some_and(|item| item.probability_bps >= wanted.probability_bps)
    })
}

fn upper_no_larger(left: Option<u64>, right: Option<u64>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => left <= right,
        (Some(_), None) | (None, None) => true,
        (None, Some(_)) => false,
    }
}

fn merge_epsilon(plans: Vec<RetrievalPlan>, epsilon: EpsilonBuckets) -> Vec<RetrievalPlan> {
    if epsilon == EpsilonBuckets::disabled() {
        return plans;
    }
    let mut retained = Vec::new();
    for plan in plans {
        if !retained
            .iter()
            .any(|other| same_bucket(other, &plan, epsilon))
        {
            retained.push(plan);
        }
    }
    retained
}

fn same_bucket(left: &RetrievalPlan, right: &RetrievalPlan, epsilon: EpsilonBuckets) -> bool {
    left.terminal == right.terminal
        && close(
            left.metrics.readiness_time.p95_ms,
            right.metrics.readiness_time.p95_ms,
            epsilon.readiness_time_ms,
        )
        && close(
            left.metrics.readiness_bytes,
            right.metrics.readiness_bytes,
            epsilon.readiness_bytes,
        )
        && close(
            left.metrics.ready_coverage_ms,
            right.metrics.ready_coverage_ms,
            epsilon.coverage_ms,
        )
        && close(
            left.metrics.quality.lower_micros,
            right.metrics.quality.lower_micros,
            epsilon.quality_micros,
        )
}

fn close(left: u64, right: u64, epsilon: u64) -> bool {
    left.abs_diff(right) <= epsilon
}
