use crate::partial_range_store::StoreAction;
use ghostr_engine::adaptive::WholeBodyContract;
use ghostr_engine::representation::{HttpGenerationLease, TransferIdentity};

#[derive(Clone)]
pub(in crate::partial_range_store) struct SingleResponseState {
    pub(in crate::partial_range_store) owner: ResponseOwner,
    pub(in crate::partial_range_store) identity: TransferIdentity,
    pub(in crate::partial_range_store) contract: WholeBodyContract,
    pub(in crate::partial_range_store) storage: SingleResponseStorage,
    pub(in crate::partial_range_store) authority: SingleResponseAuthority,
}

#[derive(Clone, Eq, PartialEq)]
pub(in crate::partial_range_store) enum SingleResponseAuthority {
    Legacy,
    Durable(HttpGenerationLease),
    ActionScoped,
}

#[derive(Clone)]
pub(in crate::partial_range_store) enum ResponseOwner {
    Legacy(u64),
    Granted(StoreAction),
}

#[derive(Clone, Copy)]
pub(in crate::partial_range_store) enum ResponseOwnerRef<'a> {
    Legacy(u64),
    Granted(&'a StoreAction),
}

#[derive(Clone, Copy)]
pub(in crate::partial_range_store) enum SingleResponseStorage {
    Live { started: bool },
    Staged { received: u64 },
}

impl ResponseOwner {
    pub(super) fn is_active(&self) -> bool {
        match self {
            Self::Legacy(_) => true,
            Self::Granted(action) => action.is_active(),
        }
    }

    pub(super) fn matches(&self, owner: ResponseOwnerRef<'_>) -> bool {
        match (self, owner) {
            (Self::Legacy(known), ResponseOwnerRef::Legacy(seen)) => *known == seen,
            (Self::Granted(known), ResponseOwnerRef::Granted(seen)) => known.same_authority(seen),
            _ => false,
        }
    }

    pub(super) fn revoke(&self) {
        if let Self::Granted(action) = self {
            action.revoke();
        }
    }

    pub(super) fn claim_publication(&self) -> bool {
        match self {
            Self::Legacy(_) => true,
            Self::Granted(action) => action.claim_publication(),
        }
    }

    pub(super) fn as_ref(&self) -> ResponseOwnerRef<'_> {
        match self {
            Self::Legacy(id) => ResponseOwnerRef::Legacy(*id),
            Self::Granted(action) => ResponseOwnerRef::Granted(action),
        }
    }
}

impl SingleResponseAuthority {
    pub(super) fn generation(&self) -> Option<&HttpGenerationLease> {
        match self {
            Self::Durable(generation) => Some(generation),
            Self::Legacy | Self::ActionScoped => None,
        }
    }

    pub(super) fn retires_http_generation(&self) -> bool {
        matches!(self, Self::ActionScoped)
    }
}

pub(in crate::partial_range_store) fn accepted_total(
    contract: WholeBodyContract,
    total: Option<u64>,
    complete: bool,
) -> Option<u64> {
    let total = total.filter(|total| complete && *total > 0)?;
    match contract {
        WholeBodyContract::Exact { expected_bytes } if total == expected_bytes => Some(total),
        WholeBodyContract::Capped { maximum_bytes } if total <= maximum_bytes => Some(total),
        _ => None,
    }
}
