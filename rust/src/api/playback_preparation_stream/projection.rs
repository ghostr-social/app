use super::PreparationContext;
use crate::api::delivery_types::FfiPlaybackPreparationPlan;
use ghostr_delivery::delivery_events::PlanEvidence;
use ghostr_delivery::startup_certificate::StartupCertificate;
use ghostr_engine::adaptive::{
    NextReserveEvidence, ReserveCandidateEvidence, ReserveCandidateState,
};
use ghostr_engine::PostId;

mod asset;

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
    for (post, certificate) in certified_upcoming(evidence) {
        if let Some(asset) = asset::project(context, post, Some(certificate)).await {
            projected.push(asset);
        }
    }
    projected
}

fn certified_upcoming(evidence: &PlanEvidence) -> Vec<(&PostId, &StartupCertificate)> {
    let certified: Vec<_> = evidence
        .plan
        .ready_reserve
        .candidates
        .iter()
        .filter_map(|candidate| certified_candidate(candidate, &evidence.startups))
        .collect();
    match certified.is_empty() {
        true => certified_next(evidence).into_iter().collect(),
        false => certified,
    }
}

fn certified_candidate<'a>(
    candidate: &'a ReserveCandidateEvidence,
    certificates: &'a [StartupCertificate],
) -> Option<(&'a PostId, &'a StartupCertificate)> {
    let startup = match &candidate.state {
        ReserveCandidateState::Ready { startup }
        | ReserveCandidateState::Structural { startup } => startup,
        _ => return None,
    };
    certificates
        .iter()
        .find(|certificate| certificate.matches(&candidate.post, startup))
        .map(|certificate| (&candidate.post, certificate))
}

fn certified_next(evidence: &PlanEvidence) -> Option<(&PostId, &StartupCertificate)> {
    let (post, startup) = match &evidence.plan.next_reserve {
        NextReserveEvidence::Ready { post, startup }
        | NextReserveEvidence::Structural { post, startup } => (post, startup),
        _ => return None,
    };
    evidence
        .startups
        .iter()
        .find(|certificate| certificate.matches(post, startup))
        .map(|certificate| (post, certificate))
}
