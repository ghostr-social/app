use super::{feasibility, WarpPlanner, WarpPlannerInput};
use crate::adaptive::{ActionNode, SearchDecision};

#[cfg(test)]
#[path = "capacity_demand/dependency_test.rs"]
mod dependency_test;

impl WarpPlanner {
    pub(super) fn additional_request_slot_demanded(
        &mut self,
        input: &WarpPlannerInput<'_>,
        frontier: &[ActionNode],
        network_bytes: u64,
    ) -> bool {
        let ceiling = input
            .snapshot
            .network
            .connection_ceiling
            .min(usize::from(u16::MAX)) as u16;
        if input.context.limits.request_tokens >= ceiling {
            return false;
        }
        let available = input.context.remaining_request_slots();
        let context = input
            .context
            .clone()
            .with_one_additional_request_slot(ceiling);
        let expanded = WarpPlannerInput::new(input.snapshot, input.base, input.origins, &context);
        let feasible = feasibility::apply(&expanded, frontier, &self.config, network_bytes);
        let (search, _) = self.search(&expanded, &feasible);
        has_marginal_demand(&search, &feasible.nodes, available)
    }
}

fn has_marginal_demand(search: &SearchDecision, nodes: &[ActionNode], available: u16) -> bool {
    independent_requests(search, nodes) > available
}

fn independent_requests(search: &SearchDecision, nodes: &[ActionNode]) -> u16 {
    search
        .chosen_plan
        .iter()
        .flat_map(|plan| &plan.action_ids)
        .filter_map(|id| nodes.iter().find(|node| node.id == *id))
        .filter(|node| node.requires.is_empty())
        .fold(0_u16, |total, node| {
            total.saturating_add(node.resources.requests)
        })
}
