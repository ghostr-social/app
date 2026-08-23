use super::{PlayerPreparation, PlayerPreparationReport, PlayerPreparationState};
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::ContentRevision;

impl PlayerPreparationReport {
    pub fn try_new(
        authority: super::PlayerPreparationAuthority,
        attempt: super::PlayerPreparationAttempt,
        sequence: u64,
        observation: super::PlayerPreparationObservation,
    ) -> Option<Self> {
        (sequence > 0).then_some(Self {
            authority,
            attempt,
            sequence,
            observation,
        })
    }

    pub fn post(&self) -> &PostId {
        &self.authority.post
    }

    pub fn binding(&self) -> &RepresentationBinding {
        &self.authority.binding
    }

    pub fn revision(&self) -> ContentRevision {
        self.authority.revision
    }

    pub fn sequence(&self) -> u64 {
        self.sequence
    }

    pub fn state(&self) -> PlayerPreparationState {
        self.observation.state
    }

    pub fn player_capability_generation(&self) -> u64 {
        self.attempt.player_capability_generation
    }

    pub fn client_epoch(&self) -> u64 {
        self.attempt.client_epoch
    }

    pub fn attempt_generation(&self) -> u64 {
        self.attempt.attempt_generation
    }

    pub fn failure_kind(&self) -> Option<&str> {
        self.observation.failure_kind.as_deref()
    }

    pub fn observed_monotonic_us(&self) -> u64 {
        self.observation.observed_monotonic_us
    }

    pub(crate) fn is_initial(&self) -> bool {
        self.sequence == 1 && self.state() == PlayerPreparationState::Initializing
    }

    pub(crate) fn is_terminal(&self) -> bool {
        matches!(
            self.state(),
            PlayerPreparationState::Failed | PlayerPreparationState::Released
        )
    }

    pub(crate) fn same_attempt_identity(&self, other: &Self) -> bool {
        self.post() == other.post()
            && self.player_capability_generation() == other.player_capability_generation()
            && self.client_epoch() == other.client_epoch()
            && self.attempt_generation() == other.attempt_generation()
    }

    pub(crate) fn same_receipt_key(&self, other: &Self) -> bool {
        self.same_attempt_identity(other) && self.sequence() == other.sequence()
    }

    pub(crate) fn release_for_replacement(&self, replacement: &Self) -> Option<Self> {
        Some(Self {
            authority: self.authority.clone(),
            attempt: self.attempt,
            sequence: self.sequence.checked_add(1)?,
            observation: super::PlayerPreparationObservation {
                state: PlayerPreparationState::Released,
                failure_kind: None,
                observed_monotonic_us: replacement.observed_monotonic_us(),
            },
        })
    }

    pub(crate) fn advances(&self, older: &Self) -> bool {
        self.sequence > older.sequence && valid_transition(older.state(), self.state())
    }

    pub(crate) fn supersedes(&self, older: &Self) -> bool {
        if self.same_attempt_identity(older)
            && (self.player_capability_generation() != older.player_capability_generation()
                || self.authority != older.authority)
        {
            return false;
        }
        self.ordering_key() > older.ordering_key()
    }

    pub(crate) fn engine_state(&self) -> PlayerPreparation {
        match self.state() {
            PlayerPreparationState::Initializing => PlayerPreparation::Initializing,
            PlayerPreparationState::Initialized => PlayerPreparation::PluginReady,
            PlayerPreparationState::FirstFrameRendered => PlayerPreparation::FirstFrameRendered,
            PlayerPreparationState::Failed => PlayerPreparation::Failed,
            PlayerPreparationState::Released => PlayerPreparation::Unverified,
        }
    }

    fn ordering_key(&self) -> (u64, u64, u64) {
        (
            self.client_epoch(),
            self.attempt_generation(),
            self.sequence,
        )
    }
}

fn valid_transition(previous: PlayerPreparationState, next: PlayerPreparationState) -> bool {
    match previous {
        PlayerPreparationState::Initializing => next != PlayerPreparationState::Initializing,
        PlayerPreparationState::Initialized => matches!(
            next,
            PlayerPreparationState::FirstFrameRendered
                | PlayerPreparationState::Failed
                | PlayerPreparationState::Released
        ),
        PlayerPreparationState::FirstFrameRendered => matches!(
            next,
            PlayerPreparationState::Failed | PlayerPreparationState::Released
        ),
        PlayerPreparationState::Failed | PlayerPreparationState::Released => false,
    }
}
