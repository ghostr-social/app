use crate::playback::{
    AdaptiveBufferPolicy, EstimateConfidence, MediaConsumption, NetworkConditions,
    PlaybackObservation, PlaybackPhase,
};
use crate::rendition::{QualitySelectionInput, Rendition, RenditionId, RenditionSet};
use core::time::Duration;

pub(crate) fn ladder() -> RenditionSet {
    RenditionSet::try_new(vec![
        rendition("high", 6_000_000),
        rendition("low", 1_000_000),
        rendition("medium", 3_000_000),
    ])
    .expect("valid rendition ladder")
}

pub(crate) fn rendition(id: &str, bitrate: u64) -> Rendition {
    Rendition::try_new(id, bitrate).expect("valid rendition")
}

pub(crate) fn id(value: &str) -> RenditionId {
    RenditionId::try_new(value).expect("valid rendition id")
}

pub(crate) fn network(
    bits_per_second: u64,
    variability_bits_per_second: u64,
    confidence: EstimateConfidence,
) -> NetworkConditions {
    NetworkConditions::new(
        bits_per_second / 8,
        variability_bits_per_second / 8,
        Duration::from_millis(100),
        confidence,
    )
}

pub(crate) fn playing_input(
    network: NetworkConditions,
    current: Option<&str>,
    buffer_seconds: u64,
    playback_rate_milli: u16,
) -> QualitySelectionInput {
    input(
        network,
        current,
        PlaybackCase::new(buffer_seconds, playback_rate_milli, PlaybackPhase::Playing),
    )
}

pub(crate) fn phase_input(
    network: NetworkConditions,
    current: Option<&str>,
    buffer_seconds: u64,
    phase: PlaybackPhase,
) -> QualitySelectionInput {
    input(
        network,
        current,
        PlaybackCase::new(buffer_seconds, 1_000, phase),
    )
}

#[derive(Clone, Copy)]
struct PlaybackCase {
    buffer_seconds: u64,
    playback_rate_milli: u16,
    phase: PlaybackPhase,
}

impl PlaybackCase {
    const fn new(buffer_seconds: u64, playback_rate_milli: u16, phase: PlaybackPhase) -> Self {
        Self {
            buffer_seconds,
            playback_rate_milli,
            phase,
        }
    }
}

fn input(
    network: NetworkConditions,
    current: Option<&str>,
    playback: PlaybackCase,
) -> QualitySelectionInput {
    let observation = PlaybackObservation::try_new(
        Duration::ZERO,
        Duration::from_secs(playback.buffer_seconds),
        playback.playback_rate_milli,
        playback.phase,
    )
    .expect("valid observation");
    let target = AdaptiveBufferPolicy::default().target(
        network,
        MediaConsumption::new(3_000_000, playback.playback_rate_milli),
    );
    QualitySelectionInput::new(network, observation, target, current.map(id))
}
