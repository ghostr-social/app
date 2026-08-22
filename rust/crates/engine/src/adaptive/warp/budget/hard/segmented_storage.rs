use super::{HardBudget, ResourceCost};
use crate::adaptive::{ActionKind, ActionNode};

pub(super) fn consume_node(budget: &mut HardBudget, node: &ActionNode) -> bool {
    let authorized = node.authorized_resources();
    if !is_hls(node) || budget.segmented_storage.is_none() {
        return budget.consume_raw(&authorized, node.request_authority());
    }
    let Some(remaining) = budget
        .segmented_storage
        .and_then(|value| value.consume(node.resources.storage_bytes))
    else {
        return false;
    };
    let cost = progressive_cost(authorized);
    if !budget.consume_raw(&cost, node.request_authority()) {
        return false;
    }
    budget.segmented_storage = Some(remaining);
    true
}

pub(super) fn route_path(
    budget: &HardBudget,
    path: &[ActionNode],
    cost: &mut ResourceCost,
) -> bool {
    let Some(available) = budget.segmented_storage else {
        return true;
    };
    let Some((progressive, segmented)) = storage_costs(path) else {
        return false;
    };
    cost.storage_bytes = progressive;
    segmented <= available.available_bytes()
}

fn storage_costs(path: &[ActionNode]) -> Option<(u64, u64)> {
    path.iter()
        .try_fold((0_u64, 0_u64), |(progressive, segmented), node| {
            if is_hls(node) {
                Some((
                    progressive,
                    segmented.checked_add(node.resources.storage_bytes)?,
                ))
            } else {
                Some((
                    progressive.checked_add(node.resources.storage_bytes)?,
                    segmented,
                ))
            }
        })
}

fn progressive_cost(mut cost: ResourceCost) -> ResourceCost {
    cost.storage_bytes = 0;
    cost
}

fn is_hls(node: &ActionNode) -> bool {
    matches!(&node.kind, ActionKind::HlsBootstrap { .. })
}
