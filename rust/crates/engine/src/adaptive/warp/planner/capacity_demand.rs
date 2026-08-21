use super::{feasibility, WarpPlanner, WarpPlannerInput};
use crate::adaptive::ActionNode;

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
        if input.context.remaining_request_slots() != 0
            || input.context.limits.request_tokens >= ceiling
        {
            return false;
        }
        let mut context = input.context.clone();
        context.limits.request_tokens =
            context.limits.request_tokens.saturating_add(1).min(ceiling);
        let expanded = WarpPlannerInput::new(input.snapshot, input.base, input.origins, &context);
        let feasible = feasibility::apply(&expanded, frontier, &self.config, network_bytes);
        // Mirror degraded least-risk selection as well as ordinary priced search.
        self.search(&expanded, &feasible)
            .action
            .is_some_and(|node| node.resources.requests > 0)
    }
}
