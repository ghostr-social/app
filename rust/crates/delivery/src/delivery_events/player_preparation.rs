use ghostr_engine::adaptive::PlayerPreparation;

const MAX_FAILURE_KIND_BYTES: usize = 128;

mod authority;
mod followup;
mod ingress;
mod report;
pub use authority::{PlayerPreparationAuthority, PlayerPreparationClaim};
pub use followup::PlayerPreparationFollowup;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayerPreparationState {
    Initializing,
    Initialized,
    FirstFrameRendered,
    Failed,
    Released,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlayerPreparationAdmission(u64);

impl PlayerPreparationAdmission {
    pub(crate) const fn new(epoch: u64) -> Self {
        Self(epoch)
    }

    pub(crate) const fn epoch(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayerPreparationIngress {
    Accepted,
    Duplicate,
    Pending,
    Stale,
    MissingInitial,
    InvalidAdmission,
    Rejected,
    Saturated,
    Closed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayerPreparationDisposition {
    Applied,
    Duplicate,
    Stale,
    MissingInitial,
    Rejected,
    Saturated,
    Unavailable,
    Closed,
    NotAdmitted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerPreparationActorOutcome {
    Applied,
    Stale,
    Rejected,
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
