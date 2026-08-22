use crate::adaptive::CurrentAuthority;
use crate::playback::{EstimateConfidence, PlaybackPhase};

pub(super) fn authority_code(value: CurrentAuthority) -> u8 {
    match value {
        CurrentAuthority::Provisional => 0,
        CurrentAuthority::Canonical => 1,
    }
}

pub(super) fn authority(value: u8) -> CurrentAuthority {
    match value {
        0 => CurrentAuthority::Provisional,
        _ => CurrentAuthority::Canonical,
    }
}

pub(super) fn confidence_code(value: EstimateConfidence) -> u8 {
    match value {
        EstimateConfidence::Low => 0,
        EstimateConfidence::Medium => 1,
        EstimateConfidence::High => 2,
    }
}

pub(super) fn confidence(value: u8) -> EstimateConfidence {
    match value {
        0 => EstimateConfidence::Low,
        1 => EstimateConfidence::Medium,
        _ => EstimateConfidence::High,
    }
}

pub(super) fn phase_code(value: PlaybackPhase) -> u8 {
    match value {
        PlaybackPhase::Starting => 0,
        PlaybackPhase::Playing => 1,
        PlaybackPhase::NetworkStalled => 2,
        PlaybackPhase::Paused => 3,
        PlaybackPhase::Ended => 4,
        PlaybackPhase::Failed => 5,
        PlaybackPhase::Inactive => 6,
    }
}

pub(super) fn phase(value: u8) -> PlaybackPhase {
    match value {
        0 => PlaybackPhase::Starting,
        1 => PlaybackPhase::Playing,
        2 => PlaybackPhase::NetworkStalled,
        3 => PlaybackPhase::Paused,
        4 => PlaybackPhase::Ended,
        5 => PlaybackPhase::Failed,
        _ => PlaybackPhase::Inactive,
    }
}
