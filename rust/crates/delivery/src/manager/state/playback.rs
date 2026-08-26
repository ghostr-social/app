use super::DeliveryState;
use crate::delivery_events::DeliveryPlayback;
use crate::playback_admission::{PlaybackAdmission, PlaybackRejection};
use ghostr_engine::playback::{PlaybackPhase, PlaybackStatus};

impl DeliveryState {
    pub(crate) fn apply_playback_at(
        &mut self,
        update: &DeliveryPlayback,
        observed_at_ms: u64,
    ) -> PlaybackAdmission {
        let post = update.session.post().clone();
        let failed = update.observation.phase() == PlaybackPhase::Failed;
        let admission = self.admit_playback(update);
        if admission.is_accepted() && failed {
            self.learn_playback_readiness(&post, false, observed_at_ms);
        }
        admission
    }

    fn admit_playback(&mut self, update: &DeliveryPlayback) -> PlaybackAdmission {
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

#[cfg(test)]
#[path = "playback_axiom_test.rs"]
pub(crate) mod axiom_test_support;
