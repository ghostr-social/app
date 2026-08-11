use super::{DebugFeedItem, FeedState};
use crate::delivery_events::{DeliveryFocus, FocusItem};
use ghostr_engine::PostId;

const BEHIND_COUNT: usize = 2;
const AHEAD_COUNT: usize = 6;

pub(super) fn delivery_focus(state: &FeedState) -> DeliveryFocus {
    let selected = selected_index(state);
    let start = selected.saturating_sub(BEHIND_COUNT);
    let end = state.items.len().min(selected + AHEAD_COUNT + 1);
    let items = state.items[start..end]
        .iter()
        .cloned()
        .map(focus_item)
        .collect();
    DeliveryFocus::compatibility(items, selected.saturating_sub(start), 0)
}

fn selected_index(state: &FeedState) -> usize {
    state
        .current_id
        .as_ref()
        .and_then(|id| state.items.iter().position(|item| &item.id == id))
        .unwrap_or(0)
}

fn focus_item(item: DebugFeedItem) -> FocusItem {
    FocusItem {
        post: PostId::new(item.id),
        meta: item.meta,
    }
}
