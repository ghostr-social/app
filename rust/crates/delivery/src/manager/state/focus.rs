use super::DeliveryState;
use crate::delivery_events::DeliveryFocus;
use ghostr_engine::focus::FocusUpdate;
use ghostr_engine::DeliveryKind;

impl DeliveryState {
    pub(crate) fn apply_focus(&mut self, update: DeliveryFocus) -> bool {
        if !self.focus_generations.accept(update.generation) {
            return false;
        }
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
        self.discard_inactive_playback();
        self.prune_scheduling_state();
        true
    }
}
