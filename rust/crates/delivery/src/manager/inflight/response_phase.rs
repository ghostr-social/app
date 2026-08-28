use crate::chunk::downloader::{OpenedResponse, ResponseObservation, ResponseWriteMode};
use ghostr_engine::adaptive::{
    PromotionGrant, PromotionOpportunity, RetrievalRequest, WholeFetchReason,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ResponsePhase {
    AwaitingHeaders,
    Observed,
    Promotable(PromotionOpportunity),
    Opened,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ResponsePromotionStage {
    NewOpportunity,
    ExistingOpportunity,
    NotPromotable,
    Rejected,
}

impl ResponsePhase {
    pub(super) fn stage(
        &mut self,
        launched: RetrievalRequest,
        request_profile: ghostr_engine::origin_model::OriginRequestProfile,
        response: &OpenedResponse,
        observed_at_ms: u64,
    ) -> ResponsePromotionStage {
        let Some(opportunity) = opportunity(launched, request_profile, response, observed_at_ms)
        else {
            if *self == Self::AwaitingHeaders {
                *self = Self::Observed;
            }
            return ResponsePromotionStage::NotPromotable;
        };
        match *self {
            Self::AwaitingHeaders => {
                *self = Self::Promotable(opportunity);
                ResponsePromotionStage::NewOpportunity
            }
            Self::Promotable(current) if current == opportunity => {
                ResponsePromotionStage::ExistingOpportunity
            }
            Self::Observed | Self::Promotable(_) | Self::Opened => ResponsePromotionStage::Rejected,
        }
    }

    pub(super) const fn opportunity(self) -> Option<PromotionOpportunity> {
        match self {
            Self::Promotable(opportunity) => Some(opportunity),
            Self::AwaitingHeaders | Self::Observed | Self::Opened => None,
        }
    }

    pub(super) fn open(&mut self) {
        *self = Self::Opened;
    }
}

fn opportunity(
    launched: RetrievalRequest,
    request_profile: ghostr_engine::origin_model::OriginRequestProfile,
    response: &OpenedResponse,
    observed_at_ms: u64,
) -> Option<PromotionOpportunity> {
    let grant = launched.promotion()?;
    if observed_at_ms > grant.valid_until_ms {
        return None;
    }
    let ResponseObservation::Body {
        request: RetrievalRequest::FetchWhole { contract, reason },
        promoted: true,
        ..
    } = response.observation()
    else {
        return None;
    };
    let valid = reason == WholeFetchReason::PromotedResponse
        && response.mode() == ResponseWriteMode::SingleResponse(contract)
        && contract.maximum_bytes() <= grant.maximum_bytes;
    valid.then_some(PromotionOpportunity::new(contract, request_profile))
}

pub(super) fn grant_matches(
    phase: ResponsePhase,
    launched: RetrievalRequest,
    target: PromotionGrant,
) -> bool {
    let (Some(latent), Some(opportunity)) = (launched.promotion(), phase.opportunity()) else {
        return false;
    };
    target.valid_until_ms == latent.valid_until_ms
        && target.maximum_bytes == opportunity.contract().maximum_bytes()
        && target.maximum_bytes <= latent.maximum_bytes
}
