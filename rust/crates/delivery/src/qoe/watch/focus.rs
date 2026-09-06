use super::{ActiveWatch, WatchCandidate};
use crate::delivery_events::{DeliveryFocus, FocusTransition, TransportRescueReason};
use core::cmp::Ordering;
use ghostr_engine::watch_model::{WatchCensor, WatchNavigation, WatchSampleKind};
use ghostr_engine::PostId;

pub(super) fn focused(focus: &DeliveryFocus) -> Option<ActiveWatch> {
    let item = focus
        .items
        .get(focus.current_index.min(focus.items.len().checked_sub(1)?))?;
    let candidate = WatchCandidate::from_item(item);
    Some(ActiveWatch {
        post: candidate.post,
        context: candidate.context,
        watched_ms: focus.watch_ms,
        terminal: false,
        generation: 0,
    })
}

pub(super) fn same_post(left: Option<&ActiveWatch>, right: Option<&ActiveWatch>) -> bool {
    left.zip(right)
        .is_some_and(|(left, right)| left.post == right.post)
}

pub(super) fn departure_kind(focus: &DeliveryFocus) -> WatchSampleKind {
    match focus.transition {
        FocusTransition::UserNavigation => WatchSampleKind::Abandoned,
        FocusTransition::RosterChange => WatchSampleKind::Censored(WatchCensor::PolicyRejection),
        FocusTransition::TransportRescue => WatchSampleKind::Censored(rescue_censor(focus)),
    }
}

fn rescue_censor(focus: &DeliveryFocus) -> WatchCensor {
    match focus.rescue.map(|rescue| rescue.reason) {
        Some(TransportRescueReason::DeliveryFailed) => WatchCensor::OriginFailure,
        _ => WatchCensor::TransportSubstitution,
    }
}

pub(super) fn navigation(previous: &PostId, focus: &DeliveryFocus) -> Option<WatchNavigation> {
    let Some(previous) = focus.items.iter().position(|item| &item.post == previous) else {
        return Some(WatchNavigation::Exit);
    };
    let current = focus.current_index.min(focus.items.len().checked_sub(1)?);
    match current.cmp(&previous) {
        Ordering::Greater => Some(WatchNavigation::Forward),
        Ordering::Less => Some(WatchNavigation::Backward),
        Ordering::Equal => None,
    }
}
