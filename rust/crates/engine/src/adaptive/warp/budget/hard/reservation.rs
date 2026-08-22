use super::{BudgetDenial, HardBudget, ResourceCost};
use crate::adaptive::ActionNode;

impl HardBudget {
    pub(in crate::adaptive::warp) fn protect(mut self, path: &[ActionNode]) -> Option<Self> {
        if !ActionNode::path_is_viable(path) || !self.can_replay(path) {
            return None;
        }
        self.pending_rescue = path.to_vec();
        Some(self)
    }

    pub(in crate::adaptive::warp) fn consume_action(
        &mut self,
        node: &ActionNode,
    ) -> Result<(), BudgetDenial> {
        let reserved = self.pending_rescue.iter().any(|item| item.id == node.id);
        if self.blocks_unreserved(node, reserved) {
            return Err(BudgetDenial::RescueReserve);
        }
        let mut updated = self.clone();
        if !super::segmented_storage::consume_node(&mut updated, node) {
            return Err(BudgetDenial::HardLimit);
        }
        if reserved {
            updated.retire_request(node);
        }
        updated.pending_rescue.retain(|item| item.id != node.id);
        let pending = updated.pending_rescue.clone();
        if !updated.can_replay(&pending) {
            return Err(BudgetDenial::RescueReserve);
        }
        *self = updated;
        Ok(())
    }

    fn blocks_unreserved(&self, node: &ActionNode, reserved: bool) -> bool {
        if reserved {
            return false;
        }
        if self
            .pending_rescue
            .iter()
            .any(|pending| pending.conflicts(node))
        {
            return true;
        }
        self.changes_rescue_authority(node)
    }

    fn changes_rescue_authority(&self, node: &ActionNode) -> bool {
        let Some(authority) = node.request_authority() else {
            return false;
        };
        self.pending_rescue.iter().any(|pending| {
            pending.resources.requests > 0 && pending.request_authority() == Some(authority)
        })
    }

    pub(in crate::adaptive::warp) fn allows_action(&self, node: &ActionNode) -> bool {
        let mut copy = self.clone();
        copy.consume_action(node).is_ok()
    }

    pub(in crate::adaptive::warp) fn allows_node(&self, node: &ActionNode) -> bool {
        let mut copy = self.clone();
        super::segmented_storage::consume_node(&mut copy, node)
    }

    pub(in crate::adaptive::warp) fn path_cost(path: &[ActionNode]) -> Option<ResourceCost> {
        path.iter()
            .try_fold(ResourceCost::default(), |total, node| {
                let resources = node.authorized_resources();
                Some(ResourceCost::new(
                    total.network_bytes.checked_add(resources.network_bytes)?,
                    total.storage_bytes.checked_add(resources.storage_bytes)?,
                    total.cpu_ms.checked_add(resources.cpu_ms)?,
                    total.requests.max(resources.requests),
                ))
            })
    }

    fn can_replay(&self, path: &[ActionNode]) -> bool {
        let Some(mut cost) = Self::path_cost(path) else {
            return false;
        };
        super::segmented_storage::route_path(self, path, &mut cost)
            && cost.no_more_than(self.remaining)
            && path.iter().all(|node| self.request_available(node))
    }

    fn request_available(&self, node: &ActionNode) -> bool {
        node.resources.requests == 0
            || node
                .request_authority()
                .is_some_and(|origin| self.origin_available(node.resources.requests, origin))
    }

    fn retire_request(&mut self, node: &ActionNode) {
        self.remaining.requests = self
            .remaining
            .requests
            .saturating_add(node.resources.requests);
        let Some(origin) = node.request_authority() else {
            return;
        };
        let remove = self.origins.get_mut(origin).is_some_and(|used| {
            *used = used.saturating_sub(usize::from(node.resources.requests));
            *used == 0
        });
        if remove {
            self.origins.remove(origin);
        }
    }
}
