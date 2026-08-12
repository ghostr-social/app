use super::PlaybackObservation;
use crate::PostId;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlaybackSession {
    post: PostId,
    generation: u64,
}

impl PlaybackSession {
    pub fn new(post: PostId, generation: u64) -> Self {
        Self { post, generation }
    }

    pub fn post(&self) -> &PostId {
        &self.post
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PlaybackObservationSequence(u64);

impl PlaybackObservationSequence {
    pub fn new(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PlaybackStatus {
    session: Option<PlaybackSession>,
    observation: Option<PlaybackObservation>,
    sequence: Option<PlaybackObservationSequence>,
    latest_generation: u64,
}

impl PlaybackStatus {
    pub fn activate(&mut self, session: PlaybackSession) -> bool {
        if self.session.as_ref() == Some(&session) {
            return true;
        }
        if !self.accepts(&session) {
            return false;
        }
        self.latest_generation = session.generation;
        self.session = Some(session);
        self.reset_evidence();
        true
    }

    pub fn discard_session(&mut self) {
        self.session = None;
        self.reset_evidence();
    }

    pub fn report(
        &mut self,
        session: &PlaybackSession,
        sequence: PlaybackObservationSequence,
        observation: PlaybackObservation,
    ) -> bool {
        if !self.matches(session) || self.sequence.is_some_and(|old| old >= sequence) {
            return false;
        }
        self.observation = Some(observation);
        self.sequence = Some(sequence);
        true
    }

    pub fn session(&self) -> Option<&PlaybackSession> {
        self.session.as_ref()
    }

    pub fn observation(&self) -> Option<PlaybackObservation> {
        self.observation
    }

    fn matches(&self, session: &PlaybackSession) -> bool {
        self.session.as_ref() == Some(session)
    }

    fn accepts(&self, session: &PlaybackSession) -> bool {
        session.generation > self.latest_generation
    }

    fn reset_evidence(&mut self) {
        self.observation = None;
        self.sequence = None;
    }
}
