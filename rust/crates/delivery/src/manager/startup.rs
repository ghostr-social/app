use crate::manager::DeliveryWorker;
use crate::startup_certificate::StartupCertificate;
use ghostr_engine::adaptive::{AllocationPlan, ReserveCandidateEvidence, ReserveCandidateState};

impl DeliveryWorker {
    pub(super) async fn startup_certificates(
        &self,
        plan: &AllocationPlan,
    ) -> Vec<StartupCertificate> {
        let mut certificates = Vec::new();
        for candidate in &plan.ready_reserve.candidates {
            if let Some(certificate) = self.startup_certificate(candidate).await {
                certificates.push(certificate);
            }
        }
        certificates
    }

    async fn startup_certificate(
        &self,
        candidate: &ReserveCandidateEvidence,
    ) -> Option<StartupCertificate> {
        let startup = match &candidate.state {
            ReserveCandidateState::Ready { startup }
            | ReserveCandidateState::Structural { startup } => startup,
            _ => return None,
        };
        let post = &candidate.post;
        let binding = self.state.catalog().binding(post)?;
        let snapshot = self.ctx.store.media_snapshot(post.as_str()).await.ok()?;
        if snapshot.binding() != Some(&binding) {
            return None;
        }
        StartupCertificate::issue(startup.clone(), &snapshot)
    }
}
