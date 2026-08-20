use crate::api::delivery::focus_mapping::validate_post_id;
use crate::api::playback_types::{
    FfiPlaybackObservation, FfiPlaybackPhase, FfiPlaybackPresentation,
};
use crate::engine::playback::{
    PlaybackObservation, PlaybackObservationSequence, PlaybackPhase, PlaybackSession,
};
use crate::engine::PostId;
use anyhow::{bail, Context, Result};
use ghostr_delivery::delivery_events::{DeliveryPlayback, PlaybackPresentation};
use std::time::Duration;

pub(crate) fn playback_update(input: FfiPlaybackObservation) -> Result<DeliveryPlayback> {
    validate_post_id(&input.post_id)?;
    if input.generation == 0 || input.sequence == 0 {
        bail!("playback generation and sequence must be positive");
    }
    let rate = u16::try_from(input.playback_rate_milli)
        .context("playback rate exceeds the supported range")?;
    let observation = PlaybackObservation::try_new(
        Duration::from_millis(input.position_ms),
        Duration::from_millis(input.buffered_extent_ms),
        rate,
        input.phase.into(),
    )
    .map_err(|error| anyhow::anyhow!("invalid playback observation: {error:?}"))?;
    Ok(DeliveryPlayback {
        session: PlaybackSession::new(PostId::new(input.post_id), input.generation),
        sequence: PlaybackObservationSequence::new(input.sequence),
        observation,
    })
}

pub(crate) fn playback_presentation(
    input: FfiPlaybackPresentation,
) -> Result<PlaybackPresentation> {
    validate_post_id(&input.post_id)?;
    if input.generation == 0 || input.sequence == 0 {
        bail!("playback presentation generation and sequence must be positive");
    }
    let session = PlaybackSession::new(PostId::new(input.post_id), input.generation);
    PlaybackPresentation::try_new(session, input.sequence, input.observed_at_ms)
        .context("playback presentation sequence must be positive")
}

impl From<FfiPlaybackPhase> for PlaybackPhase {
    fn from(phase: FfiPlaybackPhase) -> Self {
        match phase {
            FfiPlaybackPhase::Starting => Self::Starting,
            FfiPlaybackPhase::Playing => Self::Playing,
            FfiPlaybackPhase::NetworkStalled => Self::NetworkStalled,
            FfiPlaybackPhase::Paused => Self::Paused,
            FfiPlaybackPhase::Ended => Self::Ended,
            FfiPlaybackPhase::Failed => Self::Failed,
            FfiPlaybackPhase::Inactive => Self::Inactive,
        }
    }
}
