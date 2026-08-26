use crate::manager::state::DeliveryState;
use crate::manager::DeliveryWorker;
use crate::startup_certificate::StartupCertificate;
use ghostr_engine::adaptive::{AllocationPlan, ReserveCandidateEvidence, ReserveCandidateState};
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::StoredMediaSnapshot;
use std::collections::HashMap;

impl DeliveryWorker {
    pub(super) fn startup_certificates(
        &self,
        plan: &AllocationPlan,
        snapshots: &HashMap<PostId, StoredMediaSnapshot>,
    ) -> Vec<StartupCertificate> {
        startup_certificates(&self.state, plan, snapshots)
    }
}

pub(super) fn startup_certificates(
    state: &DeliveryState,
    plan: &AllocationPlan,
    snapshots: &HashMap<PostId, StoredMediaSnapshot>,
) -> Vec<StartupCertificate> {
    let mut certificates = Vec::new();
    for candidate in &plan.ready_reserve.candidates {
        if let Some(certificate) = startup_certificate(state, candidate, snapshots) {
            certificates.push(certificate);
        }
    }
    certificates
}

fn startup_certificate(
    state: &DeliveryState,
    candidate: &ReserveCandidateEvidence,
    snapshots: &HashMap<PostId, StoredMediaSnapshot>,
) -> Option<StartupCertificate> {
    let (ReserveCandidateState::Ready { startup } | ReserveCandidateState::Structural { startup }) =
        &candidate.state
    else {
        return None;
    };
    let post = &candidate.post;
    let binding = state.catalog().binding(post)?;
    let snapshot = snapshots.get(post)?;
    if !snapshot
        .binding()
        .is_some_and(|stored| stored == &binding || stored.derives_from(&binding))
    {
        return None;
    }
    StartupCertificate::issue(startup.clone(), snapshot)
}
