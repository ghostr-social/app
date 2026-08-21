use super::super::types::WarpPlannerInput;
use crate::adaptive::{HardBudget, ResourceCost};

pub(super) fn build(input: &WarpPlannerInput<'_>, network_bytes: u64) -> HardBudget {
    let active = input
        .snapshot
        .candidates
        .iter()
        .flat_map(|item| &item.in_flight)
        .filter(|item| !item.cancelling)
        .count();
    let requests = input
        .context
        .limits
        .request_tokens
        .saturating_sub(active.min(u16::MAX as usize) as u16);
    let storage = input.snapshot.storage.budget_bytes.saturating_mul(99) / 100;
    let storage = storage.saturating_sub(input.snapshot.storage.used_bytes);
    HardBudget::new(
        ResourceCost::new(
            network_bytes,
            storage,
            input.context.limits.cpu_ms,
            requests,
        ),
        input.context.limits.per_origin_requests,
    )
}
