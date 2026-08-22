use super::support::{capsule, planned, planned_network_boundary, record};
use crate::adaptive::{
    ControlMode, DecisionReplayStatus, NetworkTokenBucket, ResourcePrices, TwinConfig,
};
use crate::origin_model::OriginModel;

#[test]
fn each_full_planner_stage_is_mutation_sensitive() {
    rejects(|value| *value.origins_mut() = OriginModel::default());
    rejects(|value| value.context_mut().limits.cpu_ms = 0);
    rejects(|value| value.config_mut().twin = TwinConfig::new(1, 9_500));
    rejects(|value| value.context_mut().epochs.budget += 1);
    rejects(|value| {
        *value.controller_prices_mut() = ResourcePrices {
            request_micros: 1_000_000,
            ..ResourcePrices::default()
        };
    });
}

#[test]
fn model_epoch_is_mutation_sensitive() {
    rejects(|value| value.context_mut().epochs.model += 1);
}

#[test]
fn base_plan_is_mutation_sensitive() {
    rejects(|value| value.base_mut().mode = ControlMode::Normal);
}

#[test]
fn network_tokens_are_mutation_sensitive() {
    rejects_network(|value| mutate_network(value, (250_000, 125_000, 124_999, 10_000, 0, 0)));
}

#[test]
fn network_refill_is_mutation_sensitive() {
    rejects_network(|value| mutate_network(value, (250_000, 124_999, 125_000, 10_000, 0, 0)));
}

#[test]
fn network_update_epoch_is_mutation_sensitive() {
    rejects_network(|value| mutate_network(value, (250_000, 125_000, 125_000, 10_001, 0, 0)));
}

#[test]
fn network_debt_is_mutation_sensitive() {
    rejects_network(|value| mutate_network(value, (250_000, 125_000, 125_000, 10_000, 0, 1)));
}

#[test]
fn price_epoch_is_mutation_sensitive() {
    rejects(|value| *value.price_epoch_mut() += 1);
}

fn mutate_network(
    value: &mut crate::adaptive::PlannerReplayCapsule,
    replacement: (u64, u64, u64, u64, u64, u64),
) {
    let network = value.network_mut().expect("warm planner network state");
    assert_eq!(
        network.replay_parts(),
        (250_000, 125_000, 125_000, 10_000, 0, 0)
    );
    *network = NetworkTokenBucket::from_replay(replacement);
}

fn rejects(mutate: impl FnOnce(&mut crate::adaptive::PlannerReplayCapsule)) {
    let (state, mut decision) = planned();
    reject_mutation(state, &mut decision, mutate);
}

fn rejects_network(mutate: impl FnOnce(&mut crate::adaptive::PlannerReplayCapsule)) {
    let (state, mut decision) = planned_network_boundary();
    reject_mutation(state, &mut decision, mutate);
}

fn reject_mutation(
    state: crate::adaptive::PlayabilitySnapshot,
    decision: &mut crate::adaptive::WarpPlanningDecision,
    mutate: impl FnOnce(&mut crate::adaptive::PlannerReplayCapsule),
) {
    mutate(capsule(decision));
    let captured = record(&state, decision);

    assert_eq!(captured.replay(), DecisionReplayStatus::Verified);
    assert_eq!(
        captured.replay_warp_search(),
        Err(DecisionReplayStatus::PlanMismatch)
    );
}
