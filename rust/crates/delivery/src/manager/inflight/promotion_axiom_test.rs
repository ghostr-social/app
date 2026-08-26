use super::*;

impl PromotionTarget {
    pub(crate) fn retarget(&self, action: ActionId, identity: TransferIdentity) -> Self {
        Self::new(action, identity, self.grant)
    }
}
