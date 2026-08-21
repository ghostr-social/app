use super::{promoted_request, PromotionPreflight, PromotionRejection, PromotionTarget};
use crate::manager::inflight::ActiveChunk;

pub(super) fn active(
    current: &ActiveChunk,
    target: &PromotionTarget,
) -> Result<(), PromotionRejection> {
    identity(current, target)?;
    state(current)?;
    grant(current, target)?;
    delta(current, target)
}

pub(super) fn unchanged(current: &ActiveChunk, preflight: &PromotionPreflight) -> bool {
    active(current, &preflight.target).is_ok()
        && current.effective_request == preflight.previous_request
        && current.effective_bytes == preflight.previous_bytes
        && current.reserved_storage_bytes == preflight.previous_reservation
        && current
            .store_action
            .as_ref()
            .is_some_and(|action| action.same_authority(&preflight.store_action))
}

pub(super) fn activated(current: &ActiveChunk, preflight: &PromotionPreflight) -> bool {
    let maximum = preflight.target.grant.maximum_bytes;
    current.identity == preflight.target.identity
        && !current.response_opened
        && current.promotion_authorization == Some(preflight.target.grant)
        && current.effective_request == promoted_request(maximum)
        && current.reserved_storage_bytes == maximum
}

fn identity(current: &ActiveChunk, target: &PromotionTarget) -> Result<(), PromotionRejection> {
    if current.identity != target.identity {
        return Err(PromotionRejection::StaleIdentity);
    }
    Ok(())
}

fn state(current: &ActiveChunk) -> Result<(), PromotionRejection> {
    if current.cancelling || current.io_finished() || current.store_action.is_none() {
        return Err(PromotionRejection::Unavailable);
    }
    if current.response_opened {
        return Err(PromotionRejection::ResponseOpened);
    }
    if current.promotion_authorization.is_some() {
        return Err(PromotionRejection::AlreadyActivated);
    }
    Ok(())
}

fn grant(current: &ActiveChunk, target: &PromotionTarget) -> Result<(), PromotionRejection> {
    if current.launched_request.promotion() != Some(target.grant) {
        return Err(PromotionRejection::GrantMismatch);
    }
    Ok(())
}

fn delta(current: &ActiveChunk, target: &PromotionTarget) -> Result<(), PromotionRejection> {
    let bytes = target
        .grant
        .maximum_bytes
        .checked_sub(current.reserved_storage_bytes);
    if bytes.is_none_or(|bytes| bytes == 0) {
        return Err(PromotionRejection::InvalidDelta);
    }
    Ok(())
}
