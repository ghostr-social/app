use super::DeliveryState;
use crate::delivery_events::{DeliveryFocus, FocusPreview, FocusTransition};
use core::cmp::Ordering;
use ghostr_engine::adaptive::NavigationDirection;
use ghostr_engine::focus::FocusUpdate;
use ghostr_engine::DeliveryKind;
use std::collections::{HashMap, HashSet};

impl DeliveryState {
    pub(crate) fn current_post(&self) -> Option<ghostr_engine::PostId> {
        self.focus.current().cloned()
    }

    pub(crate) const fn focus_generation(&self) -> Option<u64> {
        self.focus_generation.value()
    }

    pub(crate) const fn focus_covers_from(&self) -> Option<u64> {
        self.focus_generation.covers_from_value()
    }

    pub(crate) fn current_authority(&self) -> ghostr_engine::adaptive::CurrentAuthority {
        self.current_authority
    }

    pub(crate) fn apply_focus(&mut self, update: DeliveryFocus, observed_at_ms: u64) -> bool {
        if !self.focus_generations.accept(update.generation) {
            return false;
        }
        let same_current =
            update.current_post().is_some() && update.current_post() == self.focus.current();
        self.reconcile_provisional_handoff(&update, same_current, observed_at_ms);
        self.focus_generation = if same_current {
            update.generation.covering(self.focus_generation)
        } else {
            update.generation
        };
        let direction = navigation_direction(self.focus.current(), &update);
        let previews = preview_map(update.previews);
        let mut window = Vec::new();
        let mut hls_focus = HashSet::new();
        for item in update.items {
            window.push(item.post.clone());
            match item.meta.delivery {
                DeliveryKind::Progressive => {
                    let preview = previews.get(&item.post).copied();
                    self.upsert_progressive(item.post.clone(), item.meta, preview);
                }
                DeliveryKind::Hls => {
                    hls_focus.insert(item.post.clone());
                    self.remove_progressive(&item.post);
                }
            }
        }
        self.hls_focus = hls_focus;
        self.current_authority = ghostr_engine::adaptive::CurrentAuthority::Canonical;
        self.focus.update_focus(FocusUpdate {
            window,
            current_index: update.current_index,
            watch_ms: update.watch_ms,
        });
        if update.transition == FocusTransition::UserNavigation {
            if let Some(direction) = direction {
                self.navigation.record(direction, observed_at_ms);
            }
        }
        self.discard_inactive_playback();
        self.prune_scheduling_state();
        true
    }
}

fn preview_map(
    previews: Vec<FocusPreview>,
) -> HashMap<ghostr_engine::PostId, ghostr_engine::PreviewDescriptor> {
    let mut mapped = HashMap::new();
    for preview in previews {
        mapped.entry(preview.post).or_insert(preview.descriptor);
    }
    mapped
}

fn navigation_direction(
    previous: Option<&ghostr_engine::PostId>,
    update: &DeliveryFocus,
) -> Option<NavigationDirection> {
    let previous = previous?;
    let previous_index = update
        .items
        .iter()
        .position(|item| &item.post == previous)?;
    let current_index = update.current_index.min(update.items.len().checked_sub(1)?);
    match current_index.cmp(&previous_index) {
        Ordering::Greater => Some(NavigationDirection::Forward),
        Ordering::Less => Some(NavigationDirection::Backward),
        Ordering::Equal => None,
    }
}
