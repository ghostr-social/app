use super::{signal, MailboxReceiver, MailboxSender};
use crate::delivery_events::{PlaybackPresentation, PlaybackPresentationIngress};
use std::collections::VecDeque;

const PLAYBACK_PRESENTATION_CAPACITY: usize = 8;

#[derive(Debug, Default)]
pub(super) struct PresentationMailbox {
    presentations: VecDeque<PlaybackPresentation>,
    latest_sequence: u64,
}

impl PresentationMailbox {
    fn insert(&mut self, event: PlaybackPresentation) -> PlaybackPresentationIngress {
        if event.sequence() <= self.latest_sequence {
            return PlaybackPresentationIngress::Stale;
        }
        if self.presentations.len() >= PLAYBACK_PRESENTATION_CAPACITY {
            return PlaybackPresentationIngress::Saturated;
        }
        self.latest_sequence = event.sequence();
        self.presentations.push_back(event);
        PlaybackPresentationIngress::Accepted
    }

    fn pop(&mut self) -> Option<PlaybackPresentation> {
        self.presentations.pop_front()
    }

    pub(super) fn clear(&mut self) {
        self.presentations.clear();
    }

    fn is_empty(&self) -> bool {
        self.presentations.is_empty()
    }
}

impl MailboxSender {
    pub(crate) fn send_playback_presentation(
        &self,
        event: PlaybackPresentation,
    ) -> PlaybackPresentationIngress {
        if self.preparation_wake.is_closed() {
            return PlaybackPresentationIngress::Closed;
        }
        let admission = self.lock().presentations.insert(event);
        if admission != PlaybackPresentationIngress::Accepted {
            return admission;
        }
        match signal(&self.preparation_wake) {
            true => admission,
            false => PlaybackPresentationIngress::Closed,
        }
    }
}

impl MailboxReceiver {
    pub(crate) fn try_playback_presentation(&mut self) -> Option<PlaybackPresentation> {
        self.lock().presentations.pop()
    }

    pub(crate) fn has_playback_presentation(&self) -> bool {
        !self.lock().presentations.is_empty()
    }
}
