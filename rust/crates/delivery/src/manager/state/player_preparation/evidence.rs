use crate::client_capability::{CapabilitySignal, ClientCapabilityProfile};
use crate::delivery_events::{
    definitive_capability_failure, PlayerPreparationReport, PlayerPreparationState,
};
use ghostr_engine::catalog::Catalog;
use ghostr_engine::evidence::{Confidence, EvidenceAssessment, EvidenceField, EvidenceValue};
use ghostr_engine::representation::{HttpGenerationStamp, RepresentationBinding};
use ghostr_engine::video_rendition::VideoRendition;
use ghostr_engine::PostId;

pub(super) fn capability_profile(
    catalog: &Catalog,
    post: &PostId,
    binding: &RepresentationBinding,
    now_ms: u64,
) -> Option<ClientCapabilityProfile> {
    let entry = catalog.lookup(post)?;
    let source = entry.meta.urls.first()?;
    let assessment = entry
        .meta
        .urls
        .first()
        .map(|url| entry.evidence_assessment_for(url, now_ms));
    let codec = assessment.as_ref().and_then(codec);
    let dimensions = assessment.as_ref().and_then(dimensions);
    let persistent = assessment
        .as_ref()
        .is_some_and(|value| verified_identity(value, entry.meta.sha256.as_deref()));
    let profile =
        ClientCapabilityProfile::try_new(binding.representation().fingerprint(), codec, dimensions)
            .ok()?;
    if persistent {
        return Some(profile.with_persistent_identity(true));
    }
    Some(profile.with_volatile_authority(post, source, volatile_generation(catalog, post, source)))
}

pub(super) fn rendition_capability_profile(
    post: &PostId,
    rendition: &VideoRendition,
) -> Option<ClientCapabilityProfile> {
    let source = rendition.meta().urls.first()?;
    ClientCapabilityProfile::try_new(rendition.identity().fingerprint(), None, None)
        .ok()
        .map(|profile| profile.with_volatile_authority(post, source, None))
}

fn verified_identity(assessment: &EvidenceAssessment, advertised: Option<&str>) -> bool {
    let Some(advertised) = advertised else {
        return false;
    };
    assessment.confidence.integrity == Confidence::certain()
        && matches!(
            assessment.value(EvidenceField::Integrity),
            Some(EvidenceValue::IntegrityMatch { digest, matches: true })
                if digest.eq_ignore_ascii_case(advertised)
        )
}

fn volatile_generation(
    catalog: &Catalog,
    post: &PostId,
    source: &str,
) -> Option<HttpGenerationStamp> {
    let identity = catalog.transfer_identity(post, source)?;
    catalog.http_generation_stamp_for(&identity)
}

pub(super) fn capability_signal(report: &PlayerPreparationReport) -> Option<CapabilitySignal> {
    match report.state() {
        PlayerPreparationState::Initializing => Some(CapabilitySignal::Initializing),
        PlayerPreparationState::FirstFrameRendered => Some(CapabilitySignal::FirstFrameRendered),
        PlayerPreparationState::Released => Some(CapabilitySignal::Released),
        PlayerPreparationState::Failed if definitive_capability_failure(report.failure_kind()) => {
            Some(CapabilitySignal::UnsupportedFailure)
        }
        PlayerPreparationState::Failed => Some(CapabilitySignal::InconclusiveFailure),
        PlayerPreparationState::Initialized => None,
    }
}

fn codec(assessment: &EvidenceAssessment) -> Option<&str> {
    match assessment.value(EvidenceField::Codec) {
        Some(EvidenceValue::Codec(value)) => Some(value),
        _ => None,
    }
}

fn dimensions(assessment: &EvidenceAssessment) -> Option<(u32, u32)> {
    match assessment.value(EvidenceField::Dimensions) {
        Some(EvidenceValue::Dimensions { width, height }) => Some((*width, *height)),
        _ => None,
    }
}
