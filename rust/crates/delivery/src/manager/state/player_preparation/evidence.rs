use crate::client_capability::CapabilitySignal;
use crate::delivery_events::{PlayerPreparationReport, PlayerPreparationState};
use ghostr_engine::evidence::{EvidenceAssessment, EvidenceField, EvidenceValue};

pub(super) fn capability_signal(report: &PlayerPreparationReport) -> Option<CapabilitySignal> {
    match report.state() {
        PlayerPreparationState::Initializing => Some(CapabilitySignal::Initializing),
        PlayerPreparationState::FirstFrameRendered => Some(CapabilitySignal::FirstFrameRendered),
        PlayerPreparationState::Released => Some(CapabilitySignal::Released),
        PlayerPreparationState::Failed if report.failure_kind() == Some("invalidVideoTrack") => {
            Some(CapabilitySignal::UnsupportedFailure)
        }
        PlayerPreparationState::Failed => Some(CapabilitySignal::InconclusiveFailure),
        PlayerPreparationState::Initialized => None,
    }
}

pub(super) fn codec(assessment: &EvidenceAssessment) -> Option<&str> {
    match assessment.value(EvidenceField::Codec) {
        Some(EvidenceValue::Codec(value)) => Some(value),
        _ => None,
    }
}

pub(super) fn dimensions(assessment: &EvidenceAssessment) -> Option<(u32, u32)> {
    match assessment.value(EvidenceField::Dimensions) {
        Some(EvidenceValue::Dimensions { width, height }) => Some((*width, *height)),
        _ => None,
    }
}
