use super::{HardBudget, ResourceCost};
use crate::RequestAuthority;

impl HardBudget {
    pub(crate) fn unlimited() -> Self {
        Self::new(
            ResourceCost::new(u64::MAX, u64::MAX, u64::MAX, u16::MAX),
            u16::MAX,
        )
    }

    pub(crate) fn consume(
        &mut self,
        cost: &ResourceCost,
        authority: Option<&RequestAuthority>,
    ) -> bool {
        self.consume_raw(cost, authority)
    }

    pub(crate) fn allows(&self, cost: &ResourceCost, authority: Option<&RequestAuthority>) -> bool {
        let mut copy = self.clone();
        copy.consume(cost, authority)
    }
}
