use super::DeliveryState;
use crate::delivery_events::DeliveryFocus;
use ghostr_engine::adaptive::NavigationDirection;
use ghostr_engine::focus::FocusUpdate;
use ghostr_engine::DeliveryKind;
use std::cmp::Ordering;

impl DeliveryState {
    pub(crate) fn apply_focus(&mut self, update: DeliveryFocus, observed_at_ms: u64) -> bool {
        if !self.focus_generations.accept(update.generation) {
            return false;
        }
        let direction = navigation_direction(self.focus.current(), &update);
        let mut window = Vec::new();
        for item in update.items {
            window.push(item.post.clone());
            if item.meta.delivery == DeliveryKind::Progressive {
                self.upsert_progressive(item.post.clone(), item.meta);
            }
        }
        self.projection_focus = false;
        self.focus.update_focus(FocusUpdate {
            window,
            current_index: update.current_index,
            watch_ms: update.watch_ms,
        });
        if let Some(direction) = direction {
            self.navigation.record(direction, observed_at_ms);
        }
        self.discard_inactive_playback();
        self.prune_scheduling_state();
        true
    }
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
