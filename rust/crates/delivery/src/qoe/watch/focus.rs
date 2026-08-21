use super::ActiveWatch;
use crate::delivery_events::{DeliveryFocus, FocusTransition, TransportRescueReason};
use ghostr_engine::watch_model::{
    WatchCensor, WatchContext, WatchKey, WatchNavigation, WatchSampleKind,
};
use ghostr_engine::PostId;
use std::cmp::Ordering;

pub(super) fn focused(focus: &DeliveryFocus) -> Option<ActiveWatch> {
    let item = focus
        .items
        .get(focus.current_index.min(focus.items.len().checked_sub(1)?))?;
    Some(ActiveWatch {
        post: item.post.clone(),
        context: WatchContext::new(
            WatchKey::digest(item.post.as_str()),
            item.meta.duration_ms.filter(|duration| *duration > 0),
        ),
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
