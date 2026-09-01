//! Immediate release of zero-body work made strictly less urgent by navigation.

use crate::delivery_events::{DeliveryFocus, FocusTransition};
use ghostr_engine::PostId;
use std::collections::HashSet;

pub(super) fn receding_future_posts(
    previous: Option<&PostId>,
    update: &DeliveryFocus,
) -> HashSet<PostId> {
    if update.transition != FocusTransition::UserNavigation {
        return HashSet::new();
    }
    let Some(previous_index) = previous.and_then(|post| position(update, post)) else {
        return HashSet::new();
    };
    let current_index = update
        .current_index
        .min(update.items.len().saturating_sub(1));
    if current_index >= previous_index {
        return HashSet::new();
    }
    update.items[previous_index.saturating_add(1)..]
        .iter()
        .map(|item| item.post.clone())
        .collect()
}

fn position(update: &DeliveryFocus, post: &PostId) -> Option<usize> {
    update.items.iter().position(|item| &item.post == post)
}
