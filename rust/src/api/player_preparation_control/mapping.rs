use crate::api::delivery::focus_mapping::validate_post_id;
use crate::api::delivery_types::{FfiPlayerPreparationReport, FfiPlayerPreparationState};
use anyhow::Context as _;
use ghostr_delivery::delivery_events::{
    PlayerPreparationAttempt, PlayerPreparationAuthority, PlayerPreparationClaim,
    PlayerPreparationFollowup, PlayerPreparationObservation, PlayerPreparationReport,
    PlayerPreparationState,
};
use ghostr_engine::PostId;

pub(super) fn map_initial(
    input: &FfiPlayerPreparationReport,
    authority: PlayerPreparationAuthority,
) -> anyhow::Result<PlayerPreparationReport> {
    PlayerPreparationReport::try_new(
        authority,
        attempt(input)?,
        input.sequence,
        observation(input)?,
    )
    .context("player preparation sequence must be positive")
}

pub(super) fn map_followup(
    input: &FfiPlayerPreparationReport,
) -> anyhow::Result<PlayerPreparationFollowup> {
    validate_post_id(&input.post_id)?;
    let claim = PlayerPreparationClaim::try_new(
        PostId::new(input.post_id.clone()),
        input.representation_id.clone(),
        &input.asset_id,
    )
    .context("player preparation claim is invalid")?;
    PlayerPreparationFollowup::try_new(claim, attempt(input)?, input.sequence, observation(input)?)
        .context("player preparation sequence must be positive")
}

fn attempt(input: &FfiPlayerPreparationReport) -> anyhow::Result<PlayerPreparationAttempt> {
    PlayerPreparationAttempt::try_new(
        input.player_capability_generation,
        input.client_epoch,
        input.attempt_generation,
    )
    .context("player preparation generations must be positive")
}

fn observation(input: &FfiPlayerPreparationReport) -> anyhow::Result<PlayerPreparationObservation> {
    PlayerPreparationObservation::try_new(
        input.state.into(),
        input.failure_kind.clone(),
        input.observed_monotonic_us,
    )
    .context("player preparation failure kind is invalid")
}

impl From<FfiPlayerPreparationState> for PlayerPreparationState {
    fn from(value: FfiPlayerPreparationState) -> Self {
        match value {
            FfiPlayerPreparationState::Initializing => Self::Initializing,
            FfiPlayerPreparationState::Initialized => Self::Initialized,
            FfiPlayerPreparationState::FirstFrameRendered => Self::FirstFrameRendered,
            FfiPlayerPreparationState::Failed => Self::Failed,
            FfiPlayerPreparationState::Released => Self::Released,
        }
    }
}
