use super::DeliveryState;
use crate::delivery_events::PlaybackPresentation;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PresentationAdmission {
    Accepted,
    Pending,
    Stale,
}

impl DeliveryState {
    pub(crate) fn apply_presentation(
        &mut self,
        event: &PlaybackPresentation,
    ) -> PresentationAdmission {
        if event.sequence() <= self.latest_presentation_sequence {
            return PresentationAdmission::Stale;
        }
        let admission = self.presentation_admission(event);
        if admission == PresentationAdmission::Stale {
            return admission;
        }
        self.latest_presentation_sequence = event.sequence();
        if admission == PresentationAdmission::Pending {
            self.pending_presentation = Some(event.clone());
        } else {
            if self.pending_matches(event.session()) {
                self.pending_presentation = None;
            }
            let post = event.session().post().clone();
            self.learn_playback_readiness(&post, true, event.observed_at_ms());
        }
        admission
    }

    pub(crate) fn take_pending_presentation(&mut self) -> Option<PlaybackPresentation> {
        let active = self.playback.session()?.clone();
        if self.pending_matches(&active) {
            let event = self.pending_presentation.take()?;
            let post = event.session().post().clone();
            self.learn_playback_readiness(&post, true, event.observed_at_ms());
            return Some(event);
        }
        if self.pending_is_stale_for(&active) {
            self.pending_presentation = None;
        }
        None
    }

    fn presentation_admission(&self, event: &PlaybackPresentation) -> PresentationAdmission {
        let Some(active) = self.playback.session() else {
            return PresentationAdmission::Pending;
        };
        if active == event.session() {
            return PresentationAdmission::Accepted;
        }
        if active.generation() < event.session().generation() {
            return PresentationAdmission::Pending;
        }
        PresentationAdmission::Stale
    }

    fn pending_matches(&self, session: &ghostr_engine::playback::PlaybackSession) -> bool {
        self.pending_presentation
            .as_ref()
            .is_some_and(|event| event.session() == session)
    }

    fn pending_is_stale_for(&self, session: &ghostr_engine::playback::PlaybackSession) -> bool {
        self.pending_presentation.as_ref().is_some_and(|event| {
            event.session().generation() < session.generation()
                || (event.session().generation() == session.generation()
                    && event.session() != session)
        })
    }
}
