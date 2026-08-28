use crate::origin_model::{OpenBodyProfile, OriginRequestProfile};
use crate::ByteRange;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct PromotionGrant {
    pub maximum_bytes: u64,
    pub valid_until_ms: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct PromotionOpportunity {
    contract: WholeBodyContract,
    trigger_profile: OpenBodyProfile,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum WholeBodyContract {
    Exact { expected_bytes: u64 },
    Capped { maximum_bytes: u64 },
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum WholeFetchReason {
    DirectCrossover,
    PromotedResponse,
    PlannedCompletion,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum RetrievalRequest {
    FetchRange {
        bytes: ByteRange,
        promotion: Option<PromotionGrant>,
    },
    FetchWhole {
        contract: WholeBodyContract,
        reason: WholeFetchReason,
    },
}

impl WholeBodyContract {
    pub fn maximum_bytes(self) -> u64 {
        match self {
            Self::Exact { expected_bytes } => expected_bytes,
            Self::Capped { maximum_bytes } => maximum_bytes,
        }
    }
}

impl PromotionOpportunity {
    pub const fn new(contract: WholeBodyContract, request_profile: OriginRequestProfile) -> Self {
        Self {
            contract,
            trigger_profile: OpenBodyProfile::from_request(request_profile),
        }
    }

    pub const fn contract(self) -> WholeBodyContract {
        self.contract
    }

    pub const fn request_profile(self, body_bytes: u64) -> OriginRequestProfile {
        self.trigger_profile.request_profile(body_bytes)
    }
}

impl RetrievalRequest {
    pub fn requested_bytes(self) -> ByteRange {
        match self {
            Self::FetchRange { bytes, .. } => bytes,
            Self::FetchWhole { contract, .. } => ByteRange::new(0, contract.maximum_bytes()),
        }
    }

    pub(super) fn reserved_coverage(self) -> ByteRange {
        match self {
            Self::FetchRange {
                promotion: Some(grant),
                ..
            } => ByteRange::new(0, grant.maximum_bytes),
            _ => self.requested_bytes(),
        }
    }

    pub fn reserved_network_bytes(self) -> u64 {
        match self {
            Self::FetchRange {
                bytes,
                promotion: None,
            } => bytes.len(),
            Self::FetchRange {
                promotion: Some(grant),
                ..
            } => grant.maximum_bytes,
            Self::FetchWhole { contract, .. } => contract.maximum_bytes(),
        }
    }

    pub fn immediate_network_bytes(self) -> u64 {
        self.requested_bytes().len()
    }

    pub fn promotion(self) -> Option<PromotionGrant> {
        match self {
            Self::FetchRange { promotion, .. } => promotion,
            Self::FetchWhole { .. } => None,
        }
    }
}
