use super::{lock, MailboxReceiver, MailboxState};
use crate::delivery_events::{DeliveryCandidate, DeliveryCommand};
use std::sync::MutexGuard;

impl MailboxReceiver {
    pub(crate) async fn changed(&mut self) -> bool {
        if self.has_ready() {
            return true;
        }
        let control_open = !self.control_wake.is_closed();
        let candidate_open = !self.candidate_wake.is_closed();
        let preparation_open = !self.preparation_wake.is_closed();
        tokio::select! {
            value = self.control_wake.recv(), if control_open => value.is_some(),
            value = self.candidate_wake.recv(), if candidate_open => value.is_some(),
            value = self.preparation_wake.recv(), if preparation_open => value.is_some(),
            else => false,
        }
    }

    pub(in crate::delivery_events) fn try_control(&self) -> Option<DeliveryCommand> {
        self.lock().controls.pop_front()
    }

    pub(in crate::delivery_events) fn try_controls_through_focus(
        &self,
    ) -> Option<Vec<DeliveryCommand>> {
        let mut state = self.lock();
        let index = state
            .controls
            .iter()
            .position(|command| matches!(command, DeliveryCommand::Focus(_)))?;
        Some(state.controls.drain(..=index).collect())
    }

    pub(in crate::delivery_events) fn try_candidate(&self) -> Option<DeliveryCandidate> {
        self.lock().candidates.pop_front()
    }

    pub(in crate::delivery_events) fn has_control(&self) -> bool {
        !self.lock().controls.is_empty()
    }

    pub(in crate::delivery_events) fn has_candidate(&self) -> bool {
        !self.lock().candidates.is_empty()
    }

    pub(in crate::delivery_events) fn clear(&self) {
        let mut state = self.lock();
        state.controls.clear();
        state.candidates.clear();
        state.preparations.clear();
        state.presentations.clear();
    }

    fn has_ready(&self) -> bool {
        self.has_control()
            || self.has_candidate()
            || self.has_player_preparation()
            || self.has_playback_presentation()
    }

    pub(super) fn lock(&self) -> MutexGuard<'_, MailboxState> {
        lock(&self.state)
    }
}

impl Drop for MailboxReceiver {
    fn drop(&mut self) {
        self.preparation_wake.close();
        self.lock().preparations.clear();
    }
}
