mod adaptation;
mod efficiency;
mod readiness;

use super::{average, unit_rate, EvaluationTracker};
use crate::evaluation::events::{BudgetMetricEvent, IntegrityMetricEvent, SemanticMetricEvent};
use crate::evaluation::types::EvaluationSnapshot;

impl EvaluationTracker {
    pub(crate) fn budget(&mut self, event: BudgetMetricEvent) {
        self.account_storage_time(event.observed_at_ms, event.stored_bytes);
        let metrics = &mut self.metrics.budget;
        metrics.observations += 1;
        metrics.instantaneous_violations += u64::from(event.instantaneous_violation);
        self.budget_network_error_sum += i128::from(event.network_target_error_bps);
        self.budget_storage_error_sum += i128::from(event.storage_target_error_bps);
        let shadow_delta = self.last_shadow_price_total.map_or(0, |previous| {
            let difference = previous.abs_diff(event.shadow_price_total_micros);
            difference.saturating_mul(10_000) / previous.max(1)
        });
        self.shadow_delta_sum += u128::from(shadow_delta.min(10_000));
        self.last_shadow_price_total = Some(event.shadow_price_total_micros);
        self.budget_qoe_sum = self
            .budget_qoe_sum
            .saturating_add(u128::from(event.qoe_micros));
        self.matched_network_bytes = self
            .matched_network_bytes
            .saturating_add(u128::from(event.matched_network_bytes));
        self.matched_storage_byte_ms = self
            .matched_storage_byte_ms
            .saturating_add(u128::from(event.matched_storage_byte_ms));
    }

    fn account_storage_time(&mut self, observed_at_ms: u64, stored_bytes: u64) {
        if let Some((previous_at_ms, previous_bytes)) = self.last_storage_observation {
            let elapsed = observed_at_ms.saturating_sub(previous_at_ms);
            let byte_ms = u128::from(previous_bytes)
                .saturating_mul(u128::from(elapsed))
                .min(u128::from(u64::MAX)) as u64;
            self.metrics.efficiency.storage_byte_ms = self
                .metrics
                .efficiency
                .storage_byte_ms
                .saturating_add(byte_ms);
        }
        self.last_storage_observation = Some((observed_at_ms, stored_bytes));
    }

    pub(crate) fn semantic(&mut self, event: SemanticMetricEvent) {
        let metrics = &mut self.metrics.semantics;
        metrics.rank_displacement = metrics
            .rank_displacement
            .saturating_add(u64::from(event.rank_displacement));
        metrics.semantic_regret_micros = metrics
            .semantic_regret_micros
            .saturating_add(event.semantic_regret_micros);
        metrics.transport_substitutions += u64::from(event.transport_substitution);
    }

    pub(crate) fn integrity(&mut self, event: IntegrityMetricEvent) {
        let metrics = &mut self.metrics.integrity;
        match event {
            IntegrityMetricEvent::HashMismatch => metrics.hash_mismatches += 1,
            IntegrityMetricEvent::StaleValidator => metrics.stale_validator_incidents += 1,
            IntegrityMetricEvent::FalseStreamability => {
                metrics.false_streamability_classifications += 1;
            }
            IntegrityMetricEvent::MetadataCalibrationError => {
                metrics.metadata_field_calibration_errors += 1;
            }
            IntegrityMetricEvent::IncorrectRangeSplicePrevented => {
                metrics.incorrect_range_splices_prevented += 1;
            }
            IntegrityMetricEvent::ParserLimitRejection => metrics.parser_limit_rejections += 1,
            IntegrityMetricEvent::SsrfOrRedirectBlock => metrics.ssrf_redirect_blocks += 1,
        }
    }
}

pub(super) fn populate(tracker: &EvaluationTracker, output: &mut EvaluationSnapshot) {
    populate_budget(tracker, output);
    readiness::populate(tracker, &mut output.readiness);
    adaptation::populate(tracker, output);
}

fn populate_budget(tracker: &EvaluationTracker, output: &mut EvaluationSnapshot) {
    let budget = &mut output.budget;
    budget.long_run_network_target_error_bps =
        average(tracker.budget_network_error_sum, budget.observations);
    budget.long_run_storage_target_error_bps =
        average(tracker.budget_storage_error_sum, budget.observations);
    let average_delta = tracker
        .shadow_delta_sum
        .checked_div(u128::from(budget.observations))
        .unwrap_or_default()
        .min(10_000) as u16;
    budget.shadow_price_stability_bps = 10_000 - average_delta;
    budget.qoe_per_matched_network_micros =
        unit_rate(tracker.budget_qoe_sum, tracker.matched_network_bytes);
    budget.qoe_per_matched_storage_micros =
        unit_rate(tracker.budget_qoe_sum, tracker.matched_storage_byte_ms);
}
