use super::PreparationContext;
use crate::api::delivery_types::FfiPlaybackPreparationPlan;
use ghostr_delivery::delivery_events::PlanEvidence;
use ghostr_delivery::startup_certificate::StartupCertificate;
use ghostr_engine::adaptive::{
    NextReserveEvidence, ReserveCandidateEvidence, ReserveCandidateState,
};
use ghostr_engine::media_timeline::StartupFootprint;
use ghostr_engine::PostId;

mod asset;

#[derive(Clone, Copy)]
pub(super) enum CertifiedReadiness<'a> {
    Structural(&'a StartupCertificate),
    Ready(&'a StartupCertificate),
}

impl<'a> CertifiedReadiness<'a> {
    fn certificate(self) -> &'a StartupCertificate {
        match self {
            Self::Structural(certificate) | Self::Ready(certificate) => certificate,
        }
    }
}

pub(crate) async fn project(context: &PreparationContext) -> Option<FfiPlaybackPreparationPlan> {
    let evidence = context.delivery.latest_plan()?;
    let current = match evidence.current.as_ref() {
        Some(post) => asset::project(context, post, None).await,
        None => None,
    };
    let upcoming = project_evidence_upcoming(context, &evidence).await;
    let next = upcoming.first().cloned();
    Some(FfiPlaybackPreparationPlan {
        revision: evidence.revision,
        current_delivery_id: evidence
            .current
            .as_ref()
            .map(|post| post.as_str().to_owned()),
        current,
        upcoming,
        next,
    })
}

async fn project_evidence_upcoming(
    context: &PreparationContext,
    evidence: &PlanEvidence,
) -> Vec<crate::api::delivery_types::FfiPlaybackPreparationAsset> {
    let mut projected = Vec::new();
    for (post, readiness) in certified_upcoming(evidence) {
        if let Some(asset) = asset::project(context, post, Some(readiness)).await {
            projected.push(asset);
        }
    }
    projected
}

fn certified_upcoming(evidence: &PlanEvidence) -> Vec<(&PostId, CertifiedReadiness<'_>)> {
    let certified: Vec<_> = evidence
        .plan
        .ready_reserve
        .candidates
        .iter()
        .filter_map(|candidate| certified_candidate(candidate, &evidence.startups))
        .collect();
    if certified.is_empty() {
        certified_next(evidence).into_iter().collect()
    } else {
        certified
    }
}

fn certified_candidate<'a>(
    candidate: &'a ReserveCandidateEvidence,
    certificates: &'a [StartupCertificate],
) -> Option<(&'a PostId, CertifiedReadiness<'a>)> {
    let readiness = match &candidate.state {
        ReserveCandidateState::Ready { startup } => {
            CertifiedReadiness::Ready(certificate(&candidate.post, startup, certificates)?)
        }
        ReserveCandidateState::Structural { startup } => {
            CertifiedReadiness::Structural(certificate(&candidate.post, startup, certificates)?)
        }
        _ => return None,
    };
    Some((&candidate.post, readiness))
}

fn certified_next(evidence: &PlanEvidence) -> Option<(&PostId, CertifiedReadiness<'_>)> {
    let (post, readiness) = match &evidence.plan.next_reserve {
        NextReserveEvidence::Ready { post, startup } => (
            post,
            CertifiedReadiness::Ready(certificate(post, startup, &evidence.startups)?),
        ),
        NextReserveEvidence::Structural { post, startup } => (
            post,
            CertifiedReadiness::Structural(certificate(post, startup, &evidence.startups)?),
        ),
        _ => return None,
    };
    Some((post, readiness))
}

fn certificate<'a>(
    post: &PostId,
    startup: &StartupFootprint,
    certificates: &'a [StartupCertificate],
) -> Option<&'a StartupCertificate> {
    certificates
        .iter()
        .find(|certificate| certificate.matches(post, startup))
}
