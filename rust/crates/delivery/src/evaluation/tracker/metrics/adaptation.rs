use super::super::{AdaptationWindow, EvaluationTracker};
use crate::evaluation::events::AdaptationMetricEvent;
use crate::evaluation::types::EvaluationSnapshot;

const ORIGIN_CAPACITY: usize = 64;

impl EvaluationTracker {
    pub fn adaptation(&mut self, event: AdaptationMetricEvent) {
        self.note_adaptation_state(&event);
        let metrics = &mut self.metrics.adaptation;
        metrics.regret_micros = metrics.regret_micros.saturating_add(event.regret_micros);
        if let Some(succeeded) = event.succeeded {
            metrics.success_predictions += 1;
            self.adaptation_success_expected += u128::from(event.predicted_success_bps);
            self.adaptation_success_observed += u64::from(succeeded);
        }
        if let Some(on_time) = event.latency_quantiles_on_time {
            metrics.latency_predictions += 1;
            metrics.quantile_predictions += 1;
            for (index, correct) in on_time.into_iter().enumerate() {
                self.adaptation_latency_correct[index] += u64::from(correct);
            }
        }
        metrics.exploration_bytes = metrics
            .exploration_bytes
            .saturating_add(event.exploration_bytes);
        metrics.failed_exploration_bytes = metrics
            .failed_exploration_bytes
            .saturating_add(event.failed_exploration_bytes);
    }

    fn note_adaptation_state(&mut self, event: &AdaptationMetricEvent) {
        let origin = self.privacy.origin(&event.origin);
        self.retain_origin_slot(&origin);
        let window = self
            .adaptation_windows
            .entry(origin)
            .or_insert(AdaptationWindow {
                started_at_ms: None,
                last_seen_at_ms: event.observed_at_ms,
            });
        let recovered = transition(window, event.observed_at_ms, event.adapting);
        if event.adapting
            && recovered.is_none()
            && window.started_at_ms == Some(event.observed_at_ms)
        {
            self.metrics.adaptation.origin_change_points += 1;
        }
        if let Some(elapsed) = recovered {
            self.recovery_latency.push(elapsed);
        }
    }

    fn retain_origin_slot(&mut self, origin: &str) {
        if self.adaptation_windows.contains_key(origin)
            || self.adaptation_windows.len() < ORIGIN_CAPACITY
        {
            return;
        }
        let oldest = self
            .adaptation_windows
            .iter()
            .min_by_key(|(_, window)| window.last_seen_at_ms)
            .map(|(key, _)| key.clone());
        if let Some(oldest) = oldest {
            self.adaptation_windows.remove(&oldest);
        }
    }
}

fn transition(window: &mut AdaptationWindow, at_ms: u64, adapting: bool) -> Option<u64> {
    window.last_seen_at_ms = at_ms;
    if adapting && window.started_at_ms.is_none() {
        window.started_at_ms = Some(at_ms);
        return None;
    }
    (!adapting)
        .then(|| window.started_at_ms.take())
        .flatten()
        .map(|started| at_ms.saturating_sub(started))
}

pub(super) fn populate(tracker: &EvaluationTracker, output: &mut EvaluationSnapshot) {
    let metrics = &mut output.adaptation;
    metrics.success_expected_bps = mean_bps(
        tracker.adaptation_success_expected,
        metrics.success_predictions,
    );
    metrics.success_observed_bps = fraction_bps(
        u128::from(tracker.adaptation_success_observed),
        metrics.success_predictions,
    );
    metrics.success_calibration_error_bps = metrics
        .success_expected_bps
        .abs_diff(metrics.success_observed_bps);
    let samples = metrics.latency_predictions;
    metrics.latency_p50_coverage_bps =
        fraction_bps(tracker.adaptation_latency_correct[0].into(), samples);
    metrics.latency_p95_coverage_bps =
        fraction_bps(tracker.adaptation_latency_correct[1].into(), samples);
    metrics.latency_p99_coverage_bps =
        fraction_bps(tracker.adaptation_latency_correct[2].into(), samples);
    metrics.quantile_coverage_bps = metrics.latency_p95_coverage_bps;
}

fn mean_bps(sum: u128, count: u64) -> u16 {
    sum.checked_div(u128::from(count))
        .unwrap_or_default()
        .min(10_000) as u16
}

fn fraction_bps(value: u128, count: u64) -> u16 {
    value
        .saturating_mul(10_000)
        .saturating_add(u128::from(count / 2))
        .checked_div(u128::from(count))
        .unwrap_or_default()
        .min(10_000) as u16
}
