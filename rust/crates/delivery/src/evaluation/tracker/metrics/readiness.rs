use super::super::{ratio, EvaluationTracker};
use crate::evaluation::events::ReadinessMetricEvent;
use crate::evaluation::types::ReadinessMetrics;

#[derive(Clone, Copy)]
struct ReadinessInterval {
    effective_at_ms: u64,
    observed_ms: u64,
    underflow_ms: u64,
}

impl EvaluationTracker {
    pub(crate) fn readiness(&mut self, event: ReadinessMetricEvent) {
        let interval = self.readiness_interval(event);
        self.account_readiness_interval(interval);
        let replenished = self.transition_underflow(event.underflow, interval.effective_at_ms);
        self.account_readiness_evidence(event);
        self.account_readiness_calibration(event);
        self.account_replenishment(event.replenished_after_ms.or(replenished));
    }

    fn readiness_interval(&mut self, event: ReadinessMetricEvent) -> ReadinessInterval {
        let previous_at_ms = self.last_readiness_at_ms.unwrap_or(event.observed_at_ms);
        let effective_at_ms = previous_at_ms.max(event.observed_at_ms);
        let inferred_ms = effective_at_ms.saturating_sub(previous_at_ms);
        let inferred_underflow_ms = if self.readiness_underflow_active {
            inferred_ms
        } else {
            0
        };
        self.last_readiness_at_ms = Some(effective_at_ms);
        ReadinessInterval {
            effective_at_ms,
            observed_ms: event.observed_ms.max(inferred_ms).max(event.underflow_ms),
            underflow_ms: event.underflow_ms.max(inferred_underflow_ms),
        }
    }

    fn account_readiness_interval(&mut self, interval: ReadinessInterval) {
        let metrics = &mut self.metrics.readiness;
        metrics.observed_ms = metrics.observed_ms.saturating_add(interval.observed_ms);
        metrics.reserve_underflow_ms = metrics
            .reserve_underflow_ms
            .saturating_add(interval.underflow_ms);
    }

    fn transition_underflow(&mut self, underflow: bool, observed_at_ms: u64) -> Option<u64> {
        let replenished = if underflow && !self.readiness_underflow_active {
            self.metrics.readiness.reserve_underflows += 1;
            self.readiness_underflow_started_at_ms = Some(observed_at_ms);
            None
        } else if !underflow && self.readiness_underflow_active {
            self.readiness_underflow_started_at_ms
                .take()
                .map(|started| observed_at_ms.saturating_sub(started))
        } else {
            None
        };
        self.readiness_underflow_active = underflow;
        replenished
    }

    fn account_readiness_evidence(&mut self, event: ReadinessMetricEvent) {
        self.readiness_observations += 1;
        let metrics = &mut self.metrics.readiness;
        metrics.probability_weighted_ready_reserve_millis = metrics
            .probability_weighted_ready_reserve_millis
            .saturating_add(event.probability_weighted_reserve_millis);
        metrics.useful_ready_coverage_ms = metrics
            .useful_ready_coverage_ms
            .saturating_add(event.ready_coverage_ms);
        metrics.protected_rescue_slot_claims += u64::from(event.protected_slot_claimed);
        metrics.protected_rescue_slot_uses += u64::from(event.protected_slot_used);
    }

    fn account_readiness_calibration(&mut self, event: ReadinessMetricEvent) {
        let (Some(predicted), Some(observed)) =
            (event.on_time_prediction_bps, event.on_time_observed)
        else {
            return;
        };
        self.metrics.readiness.on_time_readiness_samples += 1;
        self.readiness_expected += u128::from(predicted.min(10_000));
        self.readiness_observed += u64::from(observed);
    }

    fn account_replenishment(&mut self, replenished_after_ms: Option<u64>) {
        if let Some(value) = replenished_after_ms {
            self.replenish_latency.push(value);
        }
    }
}

pub(super) fn populate(tracker: &EvaluationTracker, metrics: &mut ReadinessMetrics) {
    metrics.reserve_underflow_frequency_bps =
        ratio(metrics.reserve_underflow_ms, metrics.observed_ms);
    populate_calibration(tracker, metrics);
    metrics.protected_rescue_slot_utilization_bps = ratio(
        metrics.protected_rescue_slot_uses,
        metrics.protected_rescue_slot_claims,
    );
    metrics.probability_weighted_ready_reserve_millis = event_average(
        metrics.probability_weighted_ready_reserve_millis,
        tracker.readiness_observations,
    );
    metrics.useful_ready_coverage_ms = event_average(
        metrics.useful_ready_coverage_ms,
        tracker.readiness_observations,
    );
}

fn populate_calibration(tracker: &EvaluationTracker, metrics: &mut ReadinessMetrics) {
    let samples = metrics.on_time_readiness_samples;
    if samples == 0 {
        return;
    }
    metrics.on_time_readiness_expected_bps = expected_average(tracker.readiness_expected, samples);
    metrics.on_time_readiness_observed_bps = ratio(tracker.readiness_observed, samples);
    metrics.on_time_readiness_calibration_error_bps = metrics
        .on_time_readiness_expected_bps
        .abs_diff(metrics.on_time_readiness_observed_bps);
    metrics.on_time_readiness_calibration_bps =
        10_000 - metrics.on_time_readiness_calibration_error_bps;
}

fn event_average(total: u64, count: u64) -> u64 {
    total.checked_div(count).unwrap_or_default()
}

fn expected_average(total: u128, count: u64) -> u16 {
    total
        .checked_div(u128::from(count))
        .unwrap_or_default()
        .min(10_000) as u16
}
