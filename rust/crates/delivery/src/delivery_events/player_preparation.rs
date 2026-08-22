use ghostr_engine::adaptive::PlayerPreparation;
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::ContentRevision;

const MAX_FAILURE_KIND_BYTES: usize = 128;

mod ingress;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayerPreparationState {
    Initializing,
    Initialized,
    FirstFrameRendered,
    Failed,
    Released,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerPreparationAuthority {
    post: PostId,
    binding: RepresentationBinding,
    revision: ContentRevision,
}

impl PlayerPreparationAuthority {
    pub fn try_new(
        post: PostId,
        binding: RepresentationBinding,
        revision: ContentRevision,
    ) -> Option<Self> {
        (binding.post() == &post).then_some(Self {
            post,
            binding,
            revision,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlayerPreparationAttempt {
    player_capability_generation: u64,
    client_epoch: u64,
    attempt_generation: u64,
}

impl PlayerPreparationAttempt {
    pub fn try_new(
        player_capability_generation: u64,
        client_epoch: u64,
        attempt_generation: u64,
    ) -> Option<Self> {
        (player_capability_generation > 0 && client_epoch > 0 && attempt_generation > 0).then_some(
            Self {
                player_capability_generation,
                client_epoch,
                attempt_generation,
            },
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerPreparationObservation {
    state: PlayerPreparationState,
    failure_kind: Option<String>,
    observed_monotonic_us: u64,
}

impl PlayerPreparationObservation {
    pub fn try_new(
        state: PlayerPreparationState,
        failure_kind: Option<String>,
        observed_monotonic_us: u64,
    ) -> Option<Self> {
        valid_failure(state, failure_kind.as_deref()).then_some(Self {
            state,
            failure_kind,
            observed_monotonic_us,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerPreparationReport {
    authority: PlayerPreparationAuthority,
    attempt: PlayerPreparationAttempt,
    sequence: u64,
    observation: PlayerPreparationObservation,
}

impl PlayerPreparationReport {
    pub fn try_new(
        authority: PlayerPreparationAuthority,
        attempt: PlayerPreparationAttempt,
        sequence: u64,
        observation: PlayerPreparationObservation,
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

    pub(crate) fn supersedes(&self, older: &Self) -> bool {
        if self.same_attempt_generation(older)
            && self.player_capability_generation() != older.player_capability_generation()
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

    fn same_attempt_generation(&self, other: &Self) -> bool {
        self.client_epoch() == other.client_epoch()
            && self.attempt_generation() == other.attempt_generation()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayerPreparationIngress {
    Accepted,
    Stale,
    Saturated,
    Closed,
}

fn valid_failure(state: PlayerPreparationState, failure: Option<&str>) -> bool {
    match (state, failure) {
        (PlayerPreparationState::Failed, Some(kind)) => {
            !kind.is_empty()
                && kind.len() <= MAX_FAILURE_KIND_BYTES
                && !kind.chars().any(char::is_control)
        }
        (PlayerPreparationState::Failed, None) => false,
        (_, None) => true,
        (_, Some(_)) => false,
    }
}
