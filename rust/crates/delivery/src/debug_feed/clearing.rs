use super::{delivery_focus, DebugFeed};

impl DebugFeed {
    pub fn clear(&self) {
        let focus = {
            let mut state = self.write();
            let hidden: Vec<_> = state
                .items
                .iter()
                .map(|item| item.event_id.clone())
                .collect();
            state.hidden_events.extend(hidden);
            state.items.clear();
            state.current_id = None;
            delivery_focus(&state)
        };
        self.delivery.update_focus(focus);
    }
}
