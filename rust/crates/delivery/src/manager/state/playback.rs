use super::DeliveryState;
use crate::delivery_events::DeliveryPlayback;
use ghostr_engine::playback::PlaybackStatus;

impl DeliveryState {
    pub(crate) fn apply_playback(&mut self, update: DeliveryPlayback) -> bool {
        if self.focus.current() != Some(update.session.post()) {
            return false;
        }
        if !self.playback.activate(update.session.clone()) {
            return false;
        }
        self.playback
            .report(&update.session, update.sequence, update.observation)
    }

    pub(crate) fn playback(&self) -> &PlaybackStatus {
        &self.playback
    }

    pub(super) fn discard_inactive_playback(&mut self) {
        let active = self.playback.session().map(|session| session.post());
        if active != self.focus.current() {
            self.playback.discard_session();
        }
    }
}
