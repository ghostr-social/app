mod authority;
mod chance;

use super::{RecordedSearchReplayMode, RecordedWarpSearchInput};
use crate::adaptive::{DecisionReplayStatus, RecordedResourceCost, RecordedWarpReserve};

pub(in crate::adaptive::decision) fn verify(
    input: &RecordedWarpSearchInput,
    reserve: &RecordedWarpReserve,
) -> Result<(), DecisionReplayStatus> {
    let Some(witness) = input.reserve.as_ref() else {
        return Ok(());
    };
    require(witness == reserve)?;
    verify_policy(input, reserve)?;
    if inactive(reserve) {
        return Ok(());
    }
    verify_budget(input, reserve)?;
    verify_shape(input, reserve)?;
    verify_resources(input, reserve)?;
    verify_chance(input, reserve)
}

fn verify_policy(
    input: &RecordedWarpSearchInput,
    reserve: &RecordedWarpReserve,
) -> Result<(), DecisionReplayStatus> {
    require(input.reserve_threshold_bps == reserve.chance.map(|chance| chance.threshold_bps))?;
    require(input.reserve_degraded_reason == reserve.degraded_reason)
}

fn inactive(reserve: &RecordedWarpReserve) -> bool {
    !reserve.degraded && resources_empty(reserve) && evidence_empty(reserve)
}

fn resources_empty(reserve: &RecordedWarpReserve) -> bool {
    reserve.reserved_request_slots == 0
        && reserve.reserved_network_bytes == 0
        && reserve.reserved_storage_bytes == 0
        && reserve.reserved_cpu_ms == 0
}

fn evidence_empty(reserve: &RecordedWarpReserve) -> bool {
    reserve.global_request_width == 0
        && reserve.authority_occupancy.is_empty()
        && reserve.protected_action_ids.is_empty()
        && reserve.chance.is_none()
        && reserve.degraded_reason.is_none()
}

fn verify_budget(
    input: &RecordedWarpSearchInput,
    reserve: &RecordedWarpReserve,
) -> Result<(), DecisionReplayStatus> {
    let budget = &input.budget;
    require(budget.global_request_width == Some(reserve.global_request_width))?;
    require(budget.pending_rescue_action_ids == reserve.protected_action_ids)?;
    require(authority::matches(input, reserve))
}

fn verify_shape(
    input: &RecordedWarpSearchInput,
    reserve: &RecordedWarpReserve,
) -> Result<(), DecisionReplayStatus> {
    if reserve.degraded {
        return require(
            reserve.degraded_reason.is_some()
                && reserve.protected_action_ids.is_empty()
                && reserve.chance.is_none()
                && input.mode == RecordedSearchReplayMode::LeastRisk,
        );
    }
    require(reserve.degraded_reason.is_none())?;
    require(reserve.protected_action_ids.is_empty() == reserve.chance.is_none())
}

fn verify_resources(
    input: &RecordedWarpSearchInput,
    reserve: &RecordedWarpReserve,
) -> Result<(), DecisionReplayStatus> {
    let actual = path_cost(input, &reserve.protected_action_ids)
        .ok_or(DecisionReplayStatus::PlanMismatch)?;
    let expected = RecordedResourceCost {
        network_bytes: reserve.reserved_network_bytes,
        storage_bytes: reserve.reserved_storage_bytes,
        cpu_ms: reserve.reserved_cpu_ms,
        requests: reserve.reserved_request_slots,
    };
    require(actual == expected && fits(actual, input.budget.remaining))
}

fn path_cost(input: &RecordedWarpSearchInput, ids: &[u16]) -> Option<RecordedResourceCost> {
    ids.iter().try_fold(zero_cost(), |total, id| {
        let action = input
            .actions
            .iter()
            .find(|action| action.planner_action_id == *id)?;
        add_cost(total, action.resources)
    })
}

fn add_cost(
    total: RecordedResourceCost,
    action: RecordedResourceCost,
) -> Option<RecordedResourceCost> {
    Some(RecordedResourceCost {
        network_bytes: total.network_bytes.checked_add(action.network_bytes)?,
        storage_bytes: total.storage_bytes.checked_add(action.storage_bytes)?,
        cpu_ms: total.cpu_ms.checked_add(action.cpu_ms)?,
        requests: total.requests.max(action.requests),
    })
}

fn verify_chance(
    input: &RecordedWarpSearchInput,
    reserve: &RecordedWarpReserve,
) -> Result<(), DecisionReplayStatus> {
    chance::verify(input, reserve)
}

const fn fits(cost: RecordedResourceCost, budget: RecordedResourceCost) -> bool {
    cost.network_bytes <= budget.network_bytes
        && cost.storage_bytes <= budget.storage_bytes
        && cost.cpu_ms <= budget.cpu_ms
        && cost.requests <= budget.requests
}

const fn zero_cost() -> RecordedResourceCost {
    RecordedResourceCost {
        network_bytes: 0,
        storage_bytes: 0,
        cpu_ms: 0,
        requests: 0,
    }
}

fn require(value: bool) -> Result<(), DecisionReplayStatus> {
    value
        .then_some(())
        .ok_or(DecisionReplayStatus::PlanMismatch)
}
