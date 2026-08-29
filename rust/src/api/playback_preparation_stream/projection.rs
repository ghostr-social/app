use super::PreparationContext;
use crate::api::delivery_types::FfiPlaybackPreparationPlan;
use ghostr_delivery::delivery_events::{PlanEvidence, PlayerPreparationClaim};
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
    Ready(&'a StartupCertificate, Option<&'a PlayerPreparationClaim>),
    PlayerVerified(&'a PlayerPreparationClaim),
}

impl<'a> CertifiedReadiness<'a> {
    fn certificate(self) -> Option<&'a StartupCertificate> {
        match self {
            Self::Structural(certificate) | Self::Ready(certificate, _) => Some(certificate),
            Self::PlayerVerified(_) => None,
        }
    }

    fn player_claim(self) -> Option<&'a PlayerPreparationClaim> {
        match self {
            Self::Ready(_, claim) => claim,
            Self::PlayerVerified(claim) => Some(claim),
            Self::Structural(_) => None,
        }
    }
}

pub(crate) async fn project(context: &PreparationContext) -> Option<FfiPlaybackPreparationPlan> {
    let evidence = context.delivery.latest_plan()?;
    let current = match evidence.current.as_ref() {
        Some(post) => asset::project(context, post, current_readiness(&evidence)).await,
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

fn current_readiness(evidence: &PlanEvidence) -> Option<CertifiedReadiness<'_>> {
    let post = evidence.current.as_ref()?;
    player_claim(&evidence.player_preparations, post).map(CertifiedReadiness::PlayerVerified)
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
        .filter_map(|candidate| {
            certified_candidate(candidate, &evidence.startups, &evidence.player_preparations)
        })
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
    claims: &'a [PlayerPreparationClaim],
) -> Option<(&'a PostId, CertifiedReadiness<'a>)> {
    let readiness = match &candidate.state {
        ReserveCandidateState::Ready { startup } => CertifiedReadiness::Ready(
            certificate(&candidate.post, startup, certificates)?,
            player_claim(claims, &candidate.post),
        ),
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
            CertifiedReadiness::Ready(
                certificate(post, startup, &evidence.startups)?,
                player_claim(&evidence.player_preparations, post),
            ),
        ),
        NextReserveEvidence::Structural { post, startup } => (
            post,
            CertifiedReadiness::Structural(certificate(post, startup, &evidence.startups)?),
        ),
        _ => return None,
    };
    Some((post, readiness))
}

fn player_claim<'a>(
    claims: &'a [PlayerPreparationClaim],
    post: &PostId,
) -> Option<&'a PlayerPreparationClaim> {
    claims.iter().find(|claim| claim.post() == post)
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
