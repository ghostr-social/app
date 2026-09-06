use crate::delivery_events::{DeliveryFocus, FocusItem};
use ghostr_engine::watch_model::{WatchContext, WatchKey};
use ghostr_engine::PostId;

pub(crate) struct WatchCandidate {
    pub post: PostId,
    pub context: WatchContext,
}

impl WatchCandidate {
    pub(super) fn from_item(item: &FocusItem) -> Self {
        Self {
            post: item.post.clone(),
            context: WatchContext::new(
                WatchKey::digest(item.post.as_str()),
                item.meta.duration_ms.filter(|duration| *duration > 0),
            ),
        }
    }
}

/// Ephemeral metadata in feed order, independent of the media delivery format.
#[derive(Default)]
pub(crate) struct WatchWindow {
    candidates: Vec<WatchCandidate>,
}

impl WatchWindow {
    pub(crate) const LIMIT: usize = 25;

    pub(super) fn replace(&mut self, focus: &DeliveryFocus) {
        let current = focus.current_index.min(focus.items.len().saturating_sub(1));
        self.candidates = focus
            .items
            .iter()
            .skip(current)
            .take(Self::LIMIT)
            .map(WatchCandidate::from_item)
            .collect();
    }

    pub(crate) fn for_current(&self, current: &PostId) -> Option<&[WatchCandidate]> {
        let index = self
            .candidates
            .iter()
            .position(|candidate| &candidate.post == current)?;
        Some(&self.candidates[index..])
    }
}
