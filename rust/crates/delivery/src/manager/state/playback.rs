use super::DeliveryState;
use crate::delivery_events::DeliveryPlayback;
use crate::playback_admission::{PlaybackAdmission, PlaybackRejection};
use ghostr_engine::playback::{PlaybackPhase, PlaybackStatus};

impl DeliveryState {
    pub(crate) fn apply_playback(&mut self, update: DeliveryPlayback) -> PlaybackAdmission {
        if self.focus.current() != Some(update.session.post()) {
            if update.observation.phase() == PlaybackPhase::Inactive {
                return PlaybackAdmission::IgnoredInactive;
            }
            return PlaybackAdmission::Rejected(PlaybackRejection::InactiveDelivery);
        }
        if !self.playback.activate(update.session.clone()) {
            return PlaybackAdmission::Rejected(PlaybackRejection::StaleSession);
        }
        if self
            .playback
            .report(&update.session, update.sequence, update.observation)
        {
            PlaybackAdmission::Accepted
        } else {
            PlaybackAdmission::Rejected(PlaybackRejection::StaleSequence)
        }
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
