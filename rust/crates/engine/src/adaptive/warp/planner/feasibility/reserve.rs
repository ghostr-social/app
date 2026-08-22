use super::super::types::WarpPlannerInput;
use super::rescue::RescuePlan;
use crate::adaptive::{
    ActionNode, ControlMode, HardBudget, ReserveAuthorityOccupancy, ReserveConstraint,
    ReserveDegradedReason,
};

pub(super) fn protect(
    input: &WarpPlannerInput<'_>,
    budget: HardBudget,
    rescue: Option<&RescuePlan>,
) -> (HardBudget, ReserveConstraint) {
    if input.base.mode == ControlMode::Normal {
        return (budget, ReserveConstraint::default());
    }
    let Some(plan) = rescue else {
        return (
            budget,
            degraded(input, ReserveDegradedReason::NoFeasibleRescue),
        );
    };
    match budget.clone().protect(&plan.steps) {
        Some(protected) => (protected, reserved(input, plan)),
        None => (
            budget,
            degraded(input, ReserveDegradedReason::ProtectionFailed),
        ),
    }
}

fn reserved(input: &WarpPlannerInput<'_>, plan: &RescuePlan) -> ReserveConstraint {
    ReserveConstraint {
        reserved_request_slots: plan.cost.requests,
        reserved_network_bytes: plan.cost.network_bytes,
        reserved_storage_bytes: plan.cost.storage_bytes,
        reserved_cpu_ms: plan.cost.cpu_ms,
        global_request_width: input.context.limits.request_tokens,
        authority_occupancy: authority_occupancy(input, &plan.steps),
        protected_action_ids: plan.steps.iter().map(|node| node.id).collect(),
        chance: Some(plan.chance),
        degraded: false,
        degraded_reason: None,
    }
}

fn degraded(input: &WarpPlannerInput<'_>, reason: ReserveDegradedReason) -> ReserveConstraint {
    ReserveConstraint {
        global_request_width: input.context.limits.request_tokens,
        authority_occupancy: authority_occupancy(input, &[]),
        degraded: true,
        degraded_reason: Some(reason),
        ..ReserveConstraint::default()
    }
}

fn authority_occupancy(
    input: &WarpPlannerInput<'_>,
    protected: &[ActionNode],
) -> Vec<ReserveAuthorityOccupancy> {
    let mut authorities = input.context.request_occupancy().authorities().clone();
    protected
        .iter()
        .filter_map(ActionNode::request_authority)
        .for_each(|authority| {
            authorities.entry(authority.clone()).or_default();
        });
    authorities
        .into_iter()
        .map(|(authority, occupied)| ReserveAuthorityOccupancy {
            authority,
            occupied_request_slots: occupied,
            request_width: input.context.limits.per_origin_requests,
        })
        .collect()
}
