use crate::adaptive::{
    InFlightAction, PromotionGrant, PromotionOpportunity, RetrievalRequest, WholeBodyContract,
};
use crate::origin_model::{MediaClass, OriginRequestProfile, RequestMethod};
use crate::{ActionId, ByteRange};

pub(super) fn active_range(
    action: ActionId,
    observed: Option<u64>,
    reserved_storage_bytes: Option<u64>,
    source: &str,
) -> InFlightAction {
    let mut active = InFlightAction::range(action, ByteRange::new(0, 64_000), source, 20_000, true);
    active.request = RetrievalRequest::FetchRange {
        bytes: ByteRange::new(0, 64_000),
        promotion: Some(PromotionGrant {
            maximum_bytes: 800_000,
            valid_until_ms: 20_000,
        }),
    };
    active.reserved_storage_bytes = reserved_storage_bytes.unwrap_or(64_000);
    active.promotion_opportunity = observed.map(opportunity);
    active
}

fn opportunity(expected_bytes: u64) -> PromotionOpportunity {
    let profile =
        OriginRequestProfile::new(RequestMethod::RangeGet, 64_000, MediaClass::ProgressiveMp4);
    PromotionOpportunity::new(WholeBodyContract::Exact { expected_bytes }, profile)
}
