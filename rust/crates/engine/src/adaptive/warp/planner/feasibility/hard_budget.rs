use super::super::types::WarpPlannerInput;
use crate::adaptive::{HardBudget, ResourceCost};

pub(super) fn build(input: &WarpPlannerInput<'_>, network_bytes: u64) -> HardBudget {
    let storage = input.snapshot.storage.budget_bytes.saturating_mul(99) / 100;
    let storage = storage.saturating_sub(input.snapshot.storage.used_bytes);
    HardBudget::new(
        ResourceCost::new(
            network_bytes,
            storage,
            input.context.limits.cpu_ms,
            input.context.limits.request_tokens,
        ),
        input.context.limits.per_origin_requests,
    )
    .with_occupancy(input.context.request_occupancy())
}
