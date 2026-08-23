use super::InFlightChunks;
use ghostr_engine::adaptive::{
    PromotionGrant, RetrievalRequest, WholeBodyContract, WholeFetchReason,
};
use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::{ActionId, ByteRange};
use ghostr_partial_store::partial_range_store::StoreAction;

mod validation;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PromotionTarget {
    action: ActionId,
    identity: TransferIdentity,
    grant: PromotionGrant,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PromotionRejection {
    AlreadyActivated,
    Expired,
    GrantMismatch,
    InvalidDelta,
    Missing,
    ResponseOpened,
    StaleIdentity,
    Unavailable,
}

pub(crate) struct PromotionPreflight {
    target: PromotionTarget,
    store_action: StoreAction,
    previous_request: RetrievalRequest,
    previous_bytes: ByteRange,
    previous_reservation: u64,
}

impl PromotionTarget {
    pub(crate) fn new(action: ActionId, identity: TransferIdentity, grant: PromotionGrant) -> Self {
        Self {
            action,
            identity,
            grant,
        }
    }

    pub(crate) fn maximum_bytes(&self) -> u64 {
        self.grant.maximum_bytes
    }

    #[cfg(test)]
    pub(crate) fn retarget(&self, action: ActionId, identity: TransferIdentity) -> Self {
        Self::new(action, identity, self.grant)
    }
}

impl PromotionPreflight {
    pub(crate) fn additional_bytes(&self) -> u64 {
        self.target
            .grant
            .maximum_bytes
            .saturating_sub(self.previous_reservation)
    }

    pub(crate) fn store_action(&self) -> &StoreAction {
        &self.store_action
    }
}

impl InFlightChunks {
    pub(crate) fn preflight_promotion(
        &self,
        target: &PromotionTarget,
        now_ms: u64,
    ) -> Result<PromotionPreflight, PromotionRejection> {
        if target.grant.valid_until_ms < now_ms {
            return Err(PromotionRejection::Expired);
        }
        let active = self
            .transfers
            .get(&target.action)
            .ok_or(PromotionRejection::Missing)?;
        validation::active(active, target)?;
        let store_action = active
            .store_action
            .clone()
            .ok_or(PromotionRejection::Unavailable)?;
        Ok(PromotionPreflight {
            target: target.clone(),
            store_action,
            previous_request: active.effective_request,
            previous_bytes: active.effective_bytes,
            previous_reservation: active.reserved_storage_bytes,
        })
    }

    pub(crate) fn activate_promotion(
        &mut self,
        preflight: &PromotionPreflight,
        now_ms: u64,
    ) -> bool {
        if preflight.target.grant.valid_until_ms < now_ms {
            return false;
        }
        let Some(active) = self.transfers.get_mut(&preflight.target.action) else {
            return false;
        };
        if !validation::unchanged(active, preflight) {
            return false;
        }
        let maximum_bytes = preflight.target.grant.maximum_bytes;
        active.promotion_authorization = Some(preflight.target.grant);
        active.effective_request = promoted_request(maximum_bytes);
        active.effective_bytes = ByteRange::new(0, maximum_bytes);
        active.reserved_storage_bytes = maximum_bytes;
        true
    }

    pub(crate) fn rollback_promotion(&mut self, preflight: &PromotionPreflight) -> bool {
        let Some(active) = self.transfers.get_mut(&preflight.target.action) else {
            return false;
        };
        if !validation::activated(active, preflight) {
            return false;
        }
        active.promotion_authorization = None;
        active.effective_request = preflight.previous_request;
        active.effective_bytes = preflight.previous_bytes;
        active.reserved_storage_bytes = preflight.previous_reservation;
        true
    }

    pub(crate) fn commit_promotion_network(&mut self, preflight: &PromotionPreflight) -> bool {
        let Some(active) = self.transfers.get_mut(&preflight.target.action) else {
            return false;
        };
        if !validation::activated(active, preflight) {
            return false;
        }
        let Some(committed) = active
            .committed_network_bytes
            .checked_add(preflight.additional_bytes())
        else {
            return false;
        };
        active.committed_network_bytes = committed;
        true
    }
}

pub(super) fn promoted_request(maximum_bytes: u64) -> RetrievalRequest {
    RetrievalRequest::FetchWhole {
        contract: WholeBodyContract::Capped { maximum_bytes },
        reason: WholeFetchReason::PromotedResponse,
    }
}
