use super::{CommandReceiver, DeliveryHandle};
use ghostr_engine::playback::PlaybackSession;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlaybackPresentation {
    session: PlaybackSession,
    sequence: u64,
    observed_at_ms: u64,
}

impl PlaybackPresentation {
    pub fn try_new(session: PlaybackSession, sequence: u64, observed_at_ms: u64) -> Option<Self> {
        (sequence > 0).then_some(Self {
            session,
            sequence,
            observed_at_ms,
        })
    }

    pub fn session(&self) -> &PlaybackSession {
        &self.session
    }

    pub fn sequence(&self) -> u64 {
        self.sequence
    }

    pub fn observed_at_ms(&self) -> u64 {
        self.observed_at_ms
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlaybackPresentationIngress {
    Accepted,
    Stale,
    Saturated,
    Closed,
}

impl DeliveryHandle {
    pub fn report_playback_presentation(
        &self,
        event: PlaybackPresentation,
    ) -> PlaybackPresentationIngress {
        self.sender.send_playback_presentation(event)
    }
}

impl CommandReceiver {
    pub(crate) fn try_playback_presentation(&self) -> Option<PlaybackPresentation> {
        self.commands.try_playback_presentation()
    }

    pub(crate) fn has_playback_presentation(&self) -> bool {
        self.commands.has_playback_presentation()
    }
}
