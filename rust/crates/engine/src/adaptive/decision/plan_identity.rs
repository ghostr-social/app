use super::plan;
use super::privacy::DecisionPrivacy;
use crate::adaptive::AllocationPlan;
use sha2::{Digest as _, Sha256};

const ORDERED_RESERVE_DOMAIN: &[u8] = b"ghostr-ordered-ready-reserve-v1\0";

pub(super) fn legacy(value: &AllocationPlan) -> String {
    hash(value, None)
}

pub(super) fn capture_ordered(value: &AllocationPlan, privacy: &DecisionPrivacy) -> String {
    ordered(&plan::sanitized(value, privacy))
}

pub(super) fn ordered(value: &AllocationPlan) -> String {
    hash(value, Some(ORDERED_RESERVE_DOMAIN))
}

fn hash(value: &AllocationPlan, domain: Option<&[u8]>) -> String {
    let mut digest = Sha256::new();
    if let Some(domain) = domain {
        digest.update(domain);
    }
    digest.update(format!("{value:?}").as_bytes());
    super::privacy::hex(&digest.finalize())
}
