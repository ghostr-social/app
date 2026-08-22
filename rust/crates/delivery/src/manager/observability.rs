use crate::delivery_events::{DecisionToken, LegacyDecisionPublication, WarpDecisionPublication};
use crate::evaluation::{BudgetMetricEvent, ReadinessMetricEvent, TransferMetricEvent};
use crate::manager::plan::PlannedWork;
use crate::manager::DeliveryWorker;

impl DeliveryWorker {
    pub(super) fn observe_plan(
        &self,
        planned: &PlannedWork,
        observed_at_ms: u64,
    ) -> Option<DecisionToken> {
        let snapshot = planned.snapshot.as_ref()?;
        let decision = self.publish_planning_decision(planned, snapshot);
        let evaluation = self.commands.evaluation();
        evaluation.transfer(TransferMetricEvent {
            cpu_micros: planned.planner_cpu_micros,
            ..TransferMetricEvent::default()
        });
        evaluation.budget(budget_event(planned, snapshot));
        evaluation.readiness(readiness_event(planned, observed_at_ms));
        decision
    }
}

impl DeliveryWorker {
    fn publish_planning_decision(
        &self,
        planned: &PlannedWork,
        snapshot: &ghostr_engine::adaptive::PlayabilitySnapshot,
    ) -> Option<DecisionToken> {
        match &planned.warp {
            Some(warp) => self
                .commands
                .publish_warp_decision(WarpDecisionPublication {
                    snapshot,
                    decision: warp,
                    legacy_prices: planned.shadow_prices,
                    models: &planned.decision_models,
                }),
            None => self.commands.publish_decision(LegacyDecisionPublication {
                snapshot,
                plan: &planned.plan,
                prices: planned.shadow_prices,
                models: &planned.decision_models,
            }),
        }
    }
}

fn budget_event(
    planned: &PlannedWork,
    snapshot: &ghostr_engine::adaptive::PlayabilitySnapshot,
) -> BudgetMetricEvent {
    let bytes = planned
        .plan
        .allocations
        .iter()
        .map(|item| item.request.reserved_network_bytes())
        .sum::<u64>();
    let active = planned.active_requests;
    BudgetMetricEvent {
        observed_at_ms: snapshot.observed_at_ms,
        stored_bytes: snapshot.storage.used_bytes,
        instantaneous_violation: bytes > snapshot.storage.available_bytes()
            || active + planned.plan.allocations.len() as u64
                > snapshot.network.connection_capacity as u64,
        network_target_error_bps: utilization(active, snapshot.network.connection_capacity as u64)
            - 10_000,
        storage_target_error_bps: utilization(
            snapshot.storage.used_bytes,
            snapshot.storage.budget_bytes,
        ) - 9_000,
        shadow_price_total_micros: shadow_total(planned),
        qoe_micros: planned
            .plan
            .ready_reserve
            .ready_coverage_ms
            .saturating_mul(1_000),
        matched_network_bytes: bytes,
        matched_storage_byte_ms: byte_millis(bytes, snapshot.commitment_ms),
    }
}

fn readiness_event(planned: &PlannedWork, observed_at_ms: u64) -> ReadinessMetricEvent {
    let reserve = &planned.plan.ready_reserve;
    let weighted = u128::from(reserve.ready_coverage_ms).saturating_mul(u128::from(
        10_000_u16.saturating_sub(reserve.underflow_risk_bps),
    )) / 10_000;
    ReadinessMetricEvent {
        observed_at_ms,
        underflow: reserve.target > reserve.ready,
        probability_weighted_reserve_millis: weighted.min(u128::from(u64::MAX)) as u64,
        ready_coverage_ms: reserve.ready_coverage_ms,
        on_time_prediction_bps: Some(10_000_u16.saturating_sub(reserve.underflow_risk_bps)),
        on_time_observed: Some(reserve.target <= reserve.ready),
        protected_slot_claimed: reserve.protected > 0,
        protected_slot_used: reserve.protected > 0 && !planned.plan.allocations.is_empty(),
        ..ReadinessMetricEvent::default()
    }
}

fn utilization(value: u64, capacity: u64) -> i32 {
    u128::from(value)
        .saturating_mul(10_000)
        .checked_div(u128::from(capacity.max(1)))
        .unwrap_or_default()
        .min(i32::MAX as u128) as i32
}

fn shadow_total(planned: &PlannedWork) -> u64 {
    let prices = planned.shadow_prices;
    prices
        .network_micros
        .saturating_add(prices.storage_time_micros)
        .saturating_add(prices.cpu_micros)
        .saturating_add(prices.request_micros)
}

fn byte_millis(bytes: u64, duration_ms: u64) -> u64 {
    u128::from(bytes)
        .saturating_mul(u128::from(duration_ms))
        .min(u128::from(u64::MAX)) as u64
}
