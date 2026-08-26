use super::{ResponseOwner, SingleResponseAuthority, SingleResponseState, SingleResponseStorage};
use ghostr_engine::adaptive::WholeBodyContract;
use ghostr_engine::representation::TransferIdentity;

pub(super) struct SingleResponseOpening<'a> {
    identity: &'a TransferIdentity,
    owner: ResponseOwner,
    contract: WholeBodyContract,
    authority: SingleResponseAuthority,
}

impl<'a> SingleResponseOpening<'a> {
    pub(super) fn new(
        identity: &'a TransferIdentity,
        owner: ResponseOwner,
        contract: WholeBodyContract,
        authority: SingleResponseAuthority,
    ) -> Self {
        Self {
            identity,
            owner,
            contract,
            authority,
        }
    }

    pub(super) const fn identity(&self) -> &TransferIdentity {
        self.identity
    }

    pub(super) const fn contract(&self) -> WholeBodyContract {
        self.contract
    }

    pub(super) fn forces_staged_storage(&self) -> bool {
        matches!(self.authority, SingleResponseAuthority::ActionScoped)
    }

    pub(super) fn matches(&self, known: &SingleResponseState) -> bool {
        known.owner.matches(self.owner.as_ref())
            && known.identity == *self.identity
            && known.contract == self.contract
            && known.authority == self.authority
    }

    pub(super) fn into_state(self, storage: SingleResponseStorage) -> SingleResponseState {
        SingleResponseState {
            owner: self.owner,
            identity: self.identity.clone(),
            contract: self.contract,
            storage,
            authority: self.authority,
        }
    }
}
